// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

//! [`WindowUDF`]: User Defined Window Functions

use crate::{
    Expr, PartitionEvaluator, PartitionEvaluatorFactory, ReturnTypeFunction, Signature,
    WindowFrame,
};
use arrow::datatypes::DataType;
use datafusion_common::Result;
use std::{
    fmt::{self, Debug, Display, Formatter},
    sync::Arc,
};

/// Logical representation of a user-defined window function (UDWF)
/// A UDWF is different from a UDF in that it is stateful across batches.
///
/// See the documetnation on [`PartitionEvaluator`] for more details
///
/// [`PartitionEvaluator`]: crate::PartitionEvaluator
#[derive(Clone)]
pub struct WindowUDF {
    /// name
    name: String,
    /// signature
    signature: Signature,
    /// Return type
    return_type: ReturnTypeFunction,
    /// Return the partition evaluator
    partition_evaluator_factory: PartitionEvaluatorFactory,
}

impl Debug for WindowUDF {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("WindowUDF")
            .field("name", &self.name)
            .field("signature", &self.signature)
            .field("return_type", &"<func>")
            .field("partition_evaluator_factory", &"<func>")
            .finish_non_exhaustive()
    }
}

/// Defines how the WindowUDF is shown to users
impl Display for WindowUDF {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl PartialEq for WindowUDF {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.signature == other.signature
    }
}

impl Eq for WindowUDF {}

impl std::hash::Hash for WindowUDF {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.signature.hash(state);
    }
}

impl WindowUDF {
    /// Create a new WindowUDF
    pub fn new(
        name: &str,
        signature: &Signature,
        return_type: &ReturnTypeFunction,
        partition_evaluator_factory: &PartitionEvaluatorFactory,
    ) -> Self {
        Self {
            name: name.to_string(),
            signature: signature.clone(),
            return_type: return_type.clone(),
            partition_evaluator_factory: partition_evaluator_factory.clone(),
        }
    }

    /// creates a [`Expr`] that calls the window function given
    /// the `partition_by`, `order_by`, and `window_frame` definition
    ///
    /// This utility allows using the UDWF without requiring access to
    /// the registry, such as with the DataFrame API.
    pub fn call(
        &self,
        args: Vec<Expr>,
        partition_by: Vec<Expr>,
        order_by: Vec<Expr>,
        window_frame: WindowFrame,
    ) -> Expr {
        let fun = crate::WindowFunctionDefinition::WindowUDF(Arc::new(self.clone()));

        Expr::WindowFunction(crate::expr::WindowFunction {
            fun,
            args,
            partition_by,
            order_by,
            window_frame,
        })
    }

    /// Returns this function's name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns this function's signature (what input types are accepted)
    pub fn signature(&self) -> &Signature {
        &self.signature
    }

    /// Return the type of the function given its input types
    pub fn return_type(&self, args: &[DataType]) -> Result<DataType> {
        // Old API returns an Arc of the datatype for some reason
        let res = (self.return_type)(args)?;
        Ok(res.as_ref().clone())
    }

    /// Return a `PartitionEvaluator` for evaluating this window function
    pub fn partition_evaluator_factory(&self) -> Result<Box<dyn PartitionEvaluator>> {
        (self.partition_evaluator_factory)()
    }
}
