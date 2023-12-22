mod handler;

use spin_sdk::http::{Request, Response, Router};
use spin_sdk::http_component;

use handler::{contract, user};

#[http_component]
async fn handle_abiui_api(req: Request) -> anyhow::Result<Response> {
    // cors
    if req.method().to_string() == "OPTIONS" {
        return Ok(Response::builder()
            .status(204)
            .header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Methods", "*")
            .header("Access-Control-Allow-Headers", "*")
            .header("Access-Control-Max-Age", "86400")
            .build());
    }

    let mut router = Router::new();
    // user routes
    router.get("/user/nonce/:wallet", user::nonce);
    router.post_async("/user/login", user::login);
    router.get("/user/me", user::me);
    // contract routes
    router.post("/contract/upload", contract::upload);
    router.delete("/contract/delete/:id", contract::delete);
    router.put("/contract/update_alias/:id/:alias", contract::update_alias);
    router.get("/contract/my_contracts", contract::my_contracts);

    Ok(router.handle(req))
}
