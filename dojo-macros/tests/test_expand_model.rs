use dojo_macros::Model;

#[test]
fn test_expand_model() {
    #[derive(Model)]
    #[dojo(name = "users", sort_keys = ["name"])]
    struct User {
        name: String,
        #[dojo(skip)]
        full_name: String,
    }
}
