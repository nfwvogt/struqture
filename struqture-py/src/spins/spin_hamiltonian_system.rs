// Copyright © 2021-2022 HQS Quantum Simulations GmbH. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
// in compliance with the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the
// License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
// express or implied. See the License for the specific language governing permissions and
// limitations under the License.

use super::SpinSystemWrapper;
use crate::fermions::FermionHamiltonianSystemWrapper;
use crate::spins::PauliProductWrapper;
use crate::{to_py_coo, PyCooMatrix};
use bincode::deserialize;
use num_complex::Complex64;
use pyo3::exceptions::{PyRuntimeError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyByteArray;
use qoqo_calculator::CalculatorComplex;
use qoqo_calculator_pyo3::CalculatorFloatWrapper;
use struqture::mappings::JordanWignerSpinToFermion;
use struqture::spins::{
    OperateOnSpins, SpinHamiltonianSystem, ToSparseMatrixOperator, ToSparseMatrixSuperOperator,
};
use struqture::StruqtureError;
use struqture::STRUQTURE_VERSION;
use struqture::{OperateOnDensityMatrix, OperateOnState};
use struqture_py_macros::{mappings, noiseless_system_wrapper};
/// These are representations of systems of spins.
///
/// SpinHamiltonianSystems are characterized by a SpinOperator to represent the hamiltonian of the spin system
/// and an optional number of spins.
///
/// Examples
/// --------
///
/// .. code-block:: python
///
///     import numpy.testing as npt
///     import scipy.sparse as sp
///     from qoqo_calculator_pyo3 import CalculatorComplex
///     from struqture_py.spins import SpinHamiltonianSystem, PauliProduct
///
///     ssystem = SpinHamiltonianSystem(2)
///     pp = PauliProduct().z(0)
///     ssystem.add_operator_product(pp, 5.0)
///     npt.assert_equal(ssystem.number_spins(), 2)
///     npt.assert_equal(ssystem.get(pp), CalculatorComplex(5))
///     npt.assert_equal(ssystem.keys(), [pp])
///     dimension = 4**ssystem.number_spins()
///     matrix = sp.coo_matrix(ssystem.sparse_matrix_superoperator_coo(), shape=(dimension, dimension))
///
#[pyclass(name = "SpinHamiltonianSystem", module = "struqture_py.spins")]
#[derive(Clone, Debug, PartialEq)]
pub struct SpinHamiltonianSystemWrapper {
    /// Internal storage of [struqture::spins::SpinHamiltonianSystem]
    pub internal: SpinHamiltonianSystem,
}

#[mappings(JordanWignerSpinToFermion)]
#[noiseless_system_wrapper(
    OperateOnSpins,
    OperateOnState,
    ToSparseMatrixOperator,
    ToSparseMatrixSuperOperator,
    OperateOnDensityMatrix,
    Calculus
)]
impl SpinHamiltonianSystemWrapper {
    /// Create an empty SpinHamiltonianSystem.
    ///
    /// Args:
    ///     number_spins (Optional[int]): The number of spins in the SpinHamiltonianSystem.
    ///
    /// Returns:
    ///     self: The new SpinHamiltonianSystem with the input number of spins.
    #[new]
    #[pyo3(signature = (number_spins = None))]
    pub fn new(number_spins: Option<usize>) -> Self {
        Self {
            internal: SpinHamiltonianSystem::new(number_spins),
        }
    }

    #[cfg(feature = "json_schema")]
    #[staticmethod]
    /// Return the JsonSchema for the json serialisation of the SpinHamiltonianSystem class.
    ///
    /// Returns:
    ///     str: The json schema serialized to json
    pub fn json_schema() -> String {
        let schema = schemars::schema_for!(SpinHamiltonianSystem);
        serde_json::to_string_pretty(&schema).expect("Unexpected failure to serialize schema")
    }

    #[cfg(feature = "json_schema")]
    /// Return the minimum version of struqture that supports this SpinHamiltonianSystem.
    ///
    /// Returns:
    ///     str: The minimum version of the struqture library to deserialize this object.
    pub fn min_supported_version(&self) -> String {
        return "1.0.0".to_string();
    }

    #[cfg(feature = "json_schema")]
    /// Returns the current version of the struqture library .
    ///
    /// Returns:
    ///     str: The current version of the library.
    #[staticmethod]
    pub fn current_version() -> String {
        return STRUQTURE_VERSION.to_string();
    }

    /// Implement `*` for SpinHamiltonianSystem and SpinHamiltonianSystem/CalculatorComplex/CalculatorFloat.
    ///
    /// Args:
    ///     value (Union[SpinHamiltonianSystem, CalculatorComplex, CalculatorFloat]): value by which to multiply the self SpinHamiltonianSystem
    ///
    /// Returns:
    ///     SpinSystem: The SpinHamiltonianSystem multiplied by the value.
    ///
    /// Raises:
    ///     ValueError: The rhs of the multiplication is neither CalculatorFloat, CalculatorComplex, nor SpinHamiltonianSystem.
    pub fn __mul__(&self, value: &PyAny) -> PyResult<SpinSystemWrapper> {
        let cf_value = qoqo_calculator_pyo3::convert_into_calculator_float(value);
        match cf_value {
            Ok(x) => Ok(SpinSystemWrapper {
                internal: self.clone().internal * CalculatorComplex::from(x),
            }),
            Err(_) => {
                let cc_value = qoqo_calculator_pyo3::convert_into_calculator_complex(value);
                match cc_value {
                    Ok(x) => Ok(SpinSystemWrapper {
                        internal: self.clone().internal * x,
                    }),
                    Err(_) => {
                        let bhs_value = Self::from_pyany(value.into());
                        match bhs_value {
                            Ok(x) => {
                                let new_self = (self.clone().internal * x).map_err(|err| {
                                    PyValueError::new_err(format!(
                                        "SpinHamiltonianSystems could not be multiplied: {:?}",
                                        err
                                    ))
                                })?;
                                Ok(SpinSystemWrapper { internal: new_self })
                            },
                            Err(err) => Err(PyValueError::new_err(format!(
                                "The rhs of the multiplication is neither CalculatorFloat, CalculatorComplex, nor SpinHamiltonianSystem: {:?}",
                                err)))
                        }
                    }
                }
            }
        }
    }
}
