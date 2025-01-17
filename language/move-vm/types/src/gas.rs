// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::views::{TypeView, ValueView};
use move_binary_format::errors::PartialVMResult;
use move_core_types::{
    gas_algebra::{InternalGas, NumArgs, NumBytes},
    identifier::Identifier,
    language_storage::ModuleId,
};

pub type FuncCallStack = Vec<Identifier>;

/// Enum of instructions that do not need extra information for gas metering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SimpleInstruction {
    Nop,
    Ret,

    BrTrue,
    BrFalse,
    Branch,

    Pop,
    LdU8,
    LdU64,
    LdU128,
    LdTrue,
    LdFalse,

    FreezeRef,
    MutBorrowLoc,
    ImmBorrowLoc,
    ImmBorrowField,
    MutBorrowField,
    ImmBorrowFieldGeneric,
    MutBorrowFieldGeneric,

    CastU8,
    CastU64,
    CastU128,

    Add,
    Sub,
    Mul,
    Mod,
    Div,

    BitOr,
    BitAnd,
    Xor,
    Shl,
    Shr,

    Or,
    And,
    Not,

    Lt,
    Gt,
    Le,
    Ge,

    Abort,
}

/// Trait that defines a generic gas meter interface, allowing clients of the Move VM to implement
/// their own metering scheme.
pub trait GasMeter {
    /// Charge an instruction and fail if not enough gas units are left.
    fn charge_simple_instr(
        &mut self,
        stack: &FuncCallStack,
        instr: SimpleInstruction,
    ) -> PartialVMResult<()>;

    fn charge_call(
        &mut self,
        stack: &FuncCallStack,
        module_id: &ModuleId,
        func_name: &str,
        args: impl ExactSizeIterator<Item = impl ValueView>,
    ) -> PartialVMResult<()>;

    fn charge_call_generic(
        &mut self,
        stack: &FuncCallStack,
        module_id: &ModuleId,
        func_name: &str,
        ty_args: impl ExactSizeIterator<Item = impl TypeView>,
        args: impl ExactSizeIterator<Item = impl ValueView>,
    ) -> PartialVMResult<()>;

    fn charge_ld_const(&mut self, stack: &FuncCallStack, size: NumBytes) -> PartialVMResult<()>;

    fn charge_copy_loc(
        &mut self,
        stack: &FuncCallStack,
        val: impl ValueView,
    ) -> PartialVMResult<()>;

    fn charge_move_loc(
        &mut self,
        stack: &FuncCallStack,
        val: impl ValueView,
    ) -> PartialVMResult<()>;

    fn charge_store_loc(
        &mut self,
        stack: &FuncCallStack,
        val: impl ValueView,
    ) -> PartialVMResult<()>;

    fn charge_pack(
        &mut self,
        stack: &FuncCallStack,
        is_generic: bool,
        args: impl ExactSizeIterator<Item = impl ValueView>,
    ) -> PartialVMResult<()>;

    fn charge_unpack(
        &mut self,
        stack: &FuncCallStack,
        is_generic: bool,
        args: impl ExactSizeIterator<Item = impl ValueView>,
    ) -> PartialVMResult<()>;

    fn charge_read_ref(
        &mut self,
        stack: &FuncCallStack,
        val: impl ValueView,
    ) -> PartialVMResult<()>;

    fn charge_write_ref(
        &mut self,
        stack: &FuncCallStack,
        val: impl ValueView,
    ) -> PartialVMResult<()>;

    fn charge_eq(
        &mut self,
        stack: &FuncCallStack,
        lhs: impl ValueView,
        rhs: impl ValueView,
    ) -> PartialVMResult<()>;

    fn charge_neq(
        &mut self,
        stack: &FuncCallStack,
        lhs: impl ValueView,
        rhs: impl ValueView,
    ) -> PartialVMResult<()>;

    fn charge_borrow_global(
        &mut self,
        stack: &FuncCallStack,
        is_mut: bool,
        is_generic: bool,
        ty: impl TypeView,
        is_success: bool,
    ) -> PartialVMResult<()>;

    fn charge_exists(
        &mut self,
        stack: &FuncCallStack,
        is_generic: bool,
        ty: impl TypeView,
        // TODO(Gas): see if we can get rid of this param
        exists: bool,
    ) -> PartialVMResult<()>;

    fn charge_move_from(
        &mut self,
        stack: &FuncCallStack,
        is_generic: bool,
        ty: impl TypeView,
        val: Option<impl ValueView>,
    ) -> PartialVMResult<()>;

    fn charge_move_to(
        &mut self,
        stack: &FuncCallStack,
        is_generic: bool,
        ty: impl TypeView,
        val: impl ValueView,
        is_success: bool,
    ) -> PartialVMResult<()>;

    fn charge_vec_pack<'a>(
        &mut self,
        stack: &FuncCallStack,
        ty: impl TypeView + 'a,
        args: impl ExactSizeIterator<Item = impl ValueView>,
    ) -> PartialVMResult<()>;

    fn charge_vec_len(&mut self, stack: &FuncCallStack, ty: impl TypeView) -> PartialVMResult<()>;

    fn charge_vec_borrow(
        &mut self,
        stack: &FuncCallStack,
        is_mut: bool,
        ty: impl TypeView,
        is_success: bool,
    ) -> PartialVMResult<()>;

    fn charge_vec_push_back(
        &mut self,
        stack: &FuncCallStack,
        ty: impl TypeView,
        val: impl ValueView,
    ) -> PartialVMResult<()>;

    fn charge_vec_pop_back(
        &mut self,
        stack: &FuncCallStack,
        ty: impl TypeView,
        val: Option<impl ValueView>,
    ) -> PartialVMResult<()>;

    // TODO(Gas): Expose the elements
    fn charge_vec_unpack(
        &mut self,
        stack: &FuncCallStack,
        ty: impl TypeView,
        expect_num_elements: NumArgs,
    ) -> PartialVMResult<()>;

    // TODO(Gas): Expose the two elements
    fn charge_vec_swap(&mut self, stack: &FuncCallStack, ty: impl TypeView) -> PartialVMResult<()>;

    /// Charges for loading a resource from storage. This is only called when the resource is not
    /// cached.
    /// - `Some(n)` means `n` bytes are loaded.
    /// - `None` means a load operation is performed but the resource does not exist.
    ///
    /// WARNING: This can be dangerous if you execute multiple user transactions in the same
    /// session -- identical transactions can have different gas costs. Use at your own risk.
    fn charge_load_resource(
        &mut self,
        stack: &FuncCallStack,
        loaded: Option<NumBytes>,
    ) -> PartialVMResult<()>;

    /// Charge for executing a native function.
    /// The cost is calculated returned by the native function implementation.
    /// Should fail if not enough gas units are left.
    ///
    /// In the future, we may want to remove this and directly pass a reference to the GasMeter
    /// instance to the native functions to allow gas to be deducted during computation.
    fn charge_native_function(
        &mut self,
        stack: &FuncCallStack,
        amount: InternalGas,
    ) -> PartialVMResult<()>;
}

/// A dummy gas meter that does not meter anything.
/// Charge operations will always succeed.
pub struct UnmeteredGasMeter;

impl GasMeter for UnmeteredGasMeter {
    fn charge_simple_instr(
        &mut self,
        _stack: &FuncCallStack,
        _instr: SimpleInstruction,
    ) -> PartialVMResult<()> {
        Ok(())
    }

    fn charge_call(
        &mut self,
        _stack: &FuncCallStack,
        _module_id: &ModuleId,
        _func_name: &str,
        _args: impl IntoIterator<Item = impl ValueView>,
    ) -> PartialVMResult<()> {
        Ok(())
    }

    fn charge_call_generic(
        &mut self,
        _stack: &FuncCallStack,
        _module_id: &ModuleId,
        _func_name: &str,
        _ty_args: impl ExactSizeIterator<Item = impl TypeView>,
        _args: impl ExactSizeIterator<Item = impl ValueView>,
    ) -> PartialVMResult<()> {
        Ok(())
    }

    fn charge_ld_const(&mut self, _stack: &FuncCallStack, _size: NumBytes) -> PartialVMResult<()> {
        Ok(())
    }

    fn charge_copy_loc(
        &mut self,
        _stack: &FuncCallStack,
        _val: impl ValueView,
    ) -> PartialVMResult<()> {
        Ok(())
    }

    fn charge_move_loc(
        &mut self,
        _stack: &FuncCallStack,
        _val: impl ValueView,
    ) -> PartialVMResult<()> {
        Ok(())
    }

    fn charge_store_loc(
        &mut self,
        _stack: &FuncCallStack,
        _val: impl ValueView,
    ) -> PartialVMResult<()> {
        Ok(())
    }

    fn charge_pack(
        &mut self,
        _stack: &FuncCallStack,
        _is_generic: bool,
        _args: impl ExactSizeIterator<Item = impl ValueView>,
    ) -> PartialVMResult<()> {
        Ok(())
    }

    fn charge_unpack(
        &mut self,
        _stack: &FuncCallStack,
        _is_generic: bool,
        _args: impl ExactSizeIterator<Item = impl ValueView>,
    ) -> PartialVMResult<()> {
        Ok(())
    }

    fn charge_read_ref(
        &mut self,
        _stack: &FuncCallStack,
        _val: impl ValueView,
    ) -> PartialVMResult<()> {
        Ok(())
    }

    fn charge_write_ref(
        &mut self,
        _stack: &FuncCallStack,
        _val: impl ValueView,
    ) -> PartialVMResult<()> {
        Ok(())
    }

    fn charge_eq(
        &mut self,
        _stack: &FuncCallStack,
        _lhs: impl ValueView,
        _rhs: impl ValueView,
    ) -> PartialVMResult<()> {
        Ok(())
    }

    fn charge_neq(
        &mut self,
        _stack: &FuncCallStack,
        _lhs: impl ValueView,
        _rhs: impl ValueView,
    ) -> PartialVMResult<()> {
        Ok(())
    }

    fn charge_borrow_global(
        &mut self,
        _stack: &FuncCallStack,
        _is_mut: bool,
        _is_generic: bool,
        _ty: impl TypeView,
        _is_success: bool,
    ) -> PartialVMResult<()> {
        Ok(())
    }

    fn charge_exists(
        &mut self,
        _stack: &FuncCallStack,
        _is_generic: bool,
        _ty: impl TypeView,
        _exists: bool,
    ) -> PartialVMResult<()> {
        Ok(())
    }

    fn charge_move_from(
        &mut self,
        _stack: &FuncCallStack,
        _is_generic: bool,
        _ty: impl TypeView,
        _val: Option<impl ValueView>,
    ) -> PartialVMResult<()> {
        Ok(())
    }

    fn charge_move_to(
        &mut self,
        _stack: &FuncCallStack,
        _is_generic: bool,
        _ty: impl TypeView,
        _val: impl ValueView,
        _is_success: bool,
    ) -> PartialVMResult<()> {
        Ok(())
    }

    fn charge_vec_pack<'a>(
        &mut self,
        _stack: &FuncCallStack,
        _ty: impl TypeView + 'a,
        _args: impl ExactSizeIterator<Item = impl ValueView>,
    ) -> PartialVMResult<()> {
        Ok(())
    }

    fn charge_vec_len(
        &mut self,
        _stack: &FuncCallStack,
        _ty: impl TypeView,
    ) -> PartialVMResult<()> {
        Ok(())
    }

    fn charge_vec_borrow(
        &mut self,
        _stack: &FuncCallStack,
        _is_mut: bool,
        _ty: impl TypeView,
        _is_success: bool,
    ) -> PartialVMResult<()> {
        Ok(())
    }

    fn charge_vec_push_back(
        &mut self,
        _stack: &FuncCallStack,
        _ty: impl TypeView,
        _val: impl ValueView,
    ) -> PartialVMResult<()> {
        Ok(())
    }

    fn charge_vec_pop_back(
        &mut self,
        _stack: &FuncCallStack,
        _ty: impl TypeView,
        _val: Option<impl ValueView>,
    ) -> PartialVMResult<()> {
        Ok(())
    }

    fn charge_vec_unpack(
        &mut self,
        _stack: &FuncCallStack,
        _ty: impl TypeView,
        _expect_num_elements: NumArgs,
    ) -> PartialVMResult<()> {
        Ok(())
    }

    fn charge_vec_swap(
        &mut self,
        _stack: &FuncCallStack,
        _ty: impl TypeView,
    ) -> PartialVMResult<()> {
        Ok(())
    }

    fn charge_load_resource(
        &mut self,
        _stack: &FuncCallStack,
        _loaded: Option<NumBytes>,
    ) -> PartialVMResult<()> {
        Ok(())
    }

    fn charge_native_function(
        &mut self,
        _stack: &FuncCallStack,
        _amount: InternalGas,
    ) -> PartialVMResult<()> {
        Ok(())
    }
}
