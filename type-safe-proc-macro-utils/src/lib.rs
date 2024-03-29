use proc_macro::TokenStream;

#[proc_macro]
pub fn getter_gen(item: TokenStream) -> TokenStream {
    let mut iter = item.into_iter();
    let name = iter.next().unwrap().to_string();
    let _comma = iter.next();
    let type_ = iter.next().unwrap().to_string();
    let _comma = iter.next();
    let cast_fun = iter.next().unwrap().to_string();
    format!(
        "
    pub fn get_{name}_or_null(&self, jvm: &'gc JVMState<'gc>) -> Option<{type_}> {{
        let maybe_null = self.normal_object.lookup_field(jvm,jvm,\"{name}\");
        if maybe_null.try_unwrap_object().is_some(){{
            if maybe_null.unwrap_object().is_some() {{
                maybe_null.{cast_fun}().into()
            }} else {{
                None
            }}
        }}else{{
            maybe_null.{cast_fun}().into()
        }}
    }}
    pub fn get_{name}(&self, jvm: &'gc JVMState<'gc>) -> {type_} {{
        self.get_{name}_or_null(jvm).unwrap()
    }}
    ",
        name = name,
        type_ = type_,
        cast_fun = cast_fun
    )
        .parse()
        .unwrap()
}