use classfile_parser::parse_class_file;
use std::fs::File;
use rust_jvm_common::test_utils::get_test_resources;
use rust_jvm_common::loading::Loader;
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::loading::ClassLoadingError;
use std::sync::Arc;
use rust_jvm_common::classfile::Classfile;
use rust_jvm_common::loading::LoaderName;

struct EmptyLoader{

}

impl Loader for EmptyLoader{
    fn initiating_loader_of(&self, _class: &ClassName) -> bool {
        unimplemented!()
    }

    fn find_representation_of(&self, _class: &ClassName) -> Result<File, ClassLoadingError> {
        unimplemented!()
    }

    fn load_class(&self, _class: &ClassName) -> Result<Arc<Classfile>, ClassLoadingError> {
        unimplemented!()
    }

    fn name(&self) -> LoaderName {
        unimplemented!()
    }

    fn pre_load(&self, _self_arc: Arc<dyn Loader + Sync + Send>, _name: &ClassName) -> Result<Arc<Classfile>, ClassLoadingError> {
        unimplemented!()
    }
}

#[test]
pub fn basic_class_file_parse() {
    let mut test_resources_path = get_test_resources();
    test_resources_path.push("Main.class");


    let _parsed = parse_class_file(File::open(test_resources_path.as_os_str()).unwrap(),Arc::new(EmptyLoader {}));
//    dbg!(parsed);
    //todo asserts
//    assert!(false);
    return;
}