use std::io::{Write};

pub fn prolog_initial_defs(w :&mut dyn Write){

}

const CLASS_IS_TYPE_SAFE: &str = "
classIsTypeSafe(Class) :-\
    classClassName(Class, Name) \
    classDefiningLoader(Class, L),\
    superclassChain(Name, L, Chain),\
    Chain \\= [],\
    classSuperClassName(Class, SuperclassName),\
    loadedClass(SuperclassName, L, Superclass),\
    classIsNotFinal(Superclass),\
    classMethods(Class, Methods),\
    checklist(methodIsTypeSafe(Class), Methods).\
\
classIsTypeSafe(Class) :-\
    classClassName(Class, 'java/lang/Object'),\
    classDefiningLoader(Class, L),\
    isBootstrapLoader(L),\
    classMethods(Class, Methods),\
    checklist(methodIsTypeSafe(Class), Methods).\
";