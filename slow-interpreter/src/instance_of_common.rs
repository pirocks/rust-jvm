use std::ops::Deref;
use runtime_class_stuff::RuntimeClass;

use crate::jvm_state::JVMState;
use crate::new_java_values::allocated_objects::AllocatedObject;

pub fn instance_of_nonnull<'gc, 'l>(jvm: &'gc JVMState<'gc>, obj: AllocatedObject<'gc, 'l>, expected_type: &RuntimeClass<'gc>) -> bool {
    match obj.runtime_class(jvm).deref() {
        RuntimeClass::Primitive(_) => {
            todo!()
        }
        RuntimeClass::Array(actual_arr_class) => {
            //If S is a class representing the array type SC[], that is, an array
            // of components of type SC, then:
            match expected_type {
                RuntimeClass::Primitive(_) => {
                    todo!()
                }
                RuntimeClass::Array(expected_arr_class) => {
                    //If T is an array type TC[], that is, an array of components of
                    // type TC, then one of the following must be true:
                    //TC and SC are the same primitive type
                    //TC and SC are reference types, and type SC can be cast to TC
                    // by these run-time rules
                    todo!()
                }
                RuntimeClass::Object(expected_obj_class) => {
                    if expected_obj_class.class_view.is_interface() {
                        //If T is an interface type, then T must be one of the interfaces
                        // implemented by arrays (JLS §4.10.3).
                        todo!()
                    } else {
                        //If T is a class type, then T must be Object
                        todo!()
                    }
                }
            }

        }
        RuntimeClass::Object(actual_obj_class) => {
            if actual_obj_class.class_view.is_interface() {
                //If S is an interface type, then:
                match expected_type {
                    RuntimeClass::Primitive(_) => {
                        todo!()
                    }
                    RuntimeClass::Array(expected_arr_class) => {
                        todo!()
                    }
                    RuntimeClass::Object(expected_obj_class) => {
                        if expected_obj_class.class_view.is_interface() {
                            //  If T is an interface type, then T must be the same interface as
                            // S or a superinterface of S.
                            todo!()
                        } else {
                            // If T is a class type, then T must be Object.
                            todo!()
                        }
                    }
                }

            } else {
                //If S is an ordinary (nonarray) class, then:
                match expected_type {
                    RuntimeClass::Primitive(_) => {
                        todo!()
                    }
                    RuntimeClass::Array(expected_arr_class) => {
                        todo!()
                    }
                    RuntimeClass::Object(expected_obj_class) => {
                        if expected_obj_class.class_view.is_interface() {
                            // If T is an interface type, then S must implement interface T.
                            todo!()
                        } else {
                            //  If T is a class type, then S must be the same class as T, or S
                            // must be a subclass of T;
                            todo!()
                        }
                    }
                }
            }
        }
    }
}


#[cfg(test)]
pub mod test {
    #[test]
    pub fn check_instanceof() {
        todo!()
    }
}
