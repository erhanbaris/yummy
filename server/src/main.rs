use rustpython_vm as vm;
use vm::VirtualMachine;

fn main() 
{
	vm::Interpreter::with_init(Default::default(), init_vm).enter(|vm| {
		let scope = vm.new_scope_with_builtins();

		let code_obj = vm.compile(
r#"print('hello world')"#,
vm::compiler::Mode::Exec,
"<embedded>".to_owned(),
) // end of the `compile` function
		    .unwrap_or_else(|err| {  // print the exception and exits the application
		    	vm.print_exception(vm.new_syntax_error(&err));
		        std::process::exit(0)
		    });
		if let Err(exception) = vm.run_code_obj(code_obj, scope)
		{
		    vm.print_exception(exception);
		}
	})
}

fn init_vm(vm: &mut VirtualMachine)
{
	vm.add_frozen(rustpython_pylib::frozen_stdlib());
}