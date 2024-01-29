use rlua::{Lua, MetaMethod, ToLua, UserData, Value};

fn main() {
    let lua = Lua::new();
    let result: rlua::Result<_> = lua.context(|context| {
        context.scope(|scope| {
            let val = Options::default();
            context.globals().set("options", val)?;
            context
                .load(
                    r#"
                print(options.a, options.b)
                options.a = 17
                print(options.a, options.b)
                "#,
                )
                .exec()?;
            println!("from rust: {:?}", val);
            // let userdata = scope.create_static_userdata(Options::default())?;
            // context.globals().set("options", userdata.clone())?;
            // context
            //     .load(
            //         r#"
            //     print(options.a, options.b)
            //     options.a = 17
            //     print(options.a, options.b)
            //     "#,
            //     )
            //     .exec()?;
            // let new_options = userdata.borrow::<Options>()?;
            // println!("from rust: {:?}", new_options);
            Ok(())
        })?;
        Ok(())
    });
    if let Err(err) = result {
        eprintln!("error: {:?}", err);
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Options {
    a: i32,
    b: String,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            a: 5,
            b: "hello".to_owned(),
        }
    }
}

impl UserData for Options {
    fn add_methods<'lua, T: rlua::UserDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_meta_method(MetaMethod::Index, |context, options, key: String| match key
            .as_str()
        {
            "a" => Ok(options.a.to_lua(context)),
            "b" => Ok(options.b.clone().to_lua(context)),
            _ => Err(rlua::Error::external(format!("unknown field '{}'", key))),
        });
        methods.add_meta_method_mut(
            MetaMethod::NewIndex,
            |context, options, (key, value): (String, Value)| {
                match key.as_str() {
                    "a" => options.a = context.unpack(value)?,
                    "b" => options.b = context.unpack(value)?,
                    _ => return Err(rlua::Error::external(format!("unknown field '{}'", key))),
                }
                Ok(())
            },
        )
    }
}
