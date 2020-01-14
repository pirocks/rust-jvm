pub struct RuntimeClass{
    pub classfile: Arc<Classfile>,
    pub loader: Arc<Loader>,
    pub static_vars: Map<String,JavaValue>

}

