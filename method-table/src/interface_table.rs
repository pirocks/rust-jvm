use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock};
use by_address::ByAddress;
use runtime_class_stuff::RuntimeClass;


#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct InterfaceID(pub u32);

pub struct InterfaceTableInner<'gc>{
    interfaces: Vec<Arc<RuntimeClass<'gc>>>,
    id: HashMap<ByAddress<Arc<RuntimeClass<'gc>>>, InterfaceID>
}

impl <'gc> InterfaceTableInner<'gc> {

    pub fn get_interface_id(&mut self, interface: Arc<RuntimeClass<'gc>>) -> InterfaceID{
        if let Some(res) = self.id.get(&ByAddress(interface.clone())){
            return *res;
        }
        assert!(interface.unwrap_class_class().class_view.is_interface());
        let new_id = InterfaceID(self.interfaces.len() as u32);
        self.id.insert(ByAddress(interface.clone()), new_id);
        self.interfaces.push(interface.clone());
        new_id
    }

    pub fn try_lookup(&self, id: InterfaceID) -> Option<Arc<RuntimeClass<'gc>>> {
        if id.0 < self.interfaces.len() as u32 {
            Some(self.interfaces[id.0 as usize].clone())
        } else {
            None
        }
    }

    pub fn lookup(&self, id: InterfaceID) -> Arc<RuntimeClass<'gc>> {
        self.try_lookup(id).unwrap()
    }
}

pub struct InterfaceTable<'gc>{
    inner: RwLock<InterfaceTableInner<'gc>>
}

impl <'gc> InterfaceTable<'gc> {
    pub fn new() -> Self{
        Self{
            inner: RwLock::new(InterfaceTableInner{ interfaces: vec![], id: Default::default() })
        }
    }

    pub fn get_interface_id(&self, interface: Arc<RuntimeClass<'gc>>) -> InterfaceID{
        self.inner.write().unwrap().get_interface_id(interface)
    }

    pub fn try_lookup(&self, id: InterfaceID) -> Option<Arc<RuntimeClass<'gc>>> {
        self.inner.read().unwrap().try_lookup(id)
    }

    pub fn lookup(&self, id: InterfaceID) -> Arc<RuntimeClass<'gc>> {
        self.try_lookup(id).unwrap()
    }
}
