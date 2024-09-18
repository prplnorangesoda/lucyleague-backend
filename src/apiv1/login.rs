// #[get("/api/v1/login")]
// async fn get_openid(data: web::Data<AppState>) -> HttpResponse {
//     log::info!("GET request at /login");
//     HttpResponse::Found()
//         .insert_header(("Location", data.into_inner().steam_auth_url.clone()))
//         .body("Redirecting...")
// }

// #[get("/api/v1/login/landing")]
// pub async fn openid_landing(
//     query: web::Query<HashMap<String, String>>,
//     state: web::Data<AppState>,
// ) -> Result<impl Responder, Error> {
//     log::info!("GET request at /login/landing");
//     let inner = query.into_inner();
//     log::trace!("Query parameters: {inner:?}");

//     for key in OPENID_NECESSARY_PARAMETERS {
//         // because key is &&str, we have to dereference it to a pure &str
//         // in order for it to not yell at us in compilation
//         if !inner.contains_key(*key) {
//             log::warn!("A malformed OpenId landing was received: {inner:?}");
//             return Ok(HttpResponse::BadRequest()
//                 .body("Your openid landing was malformed in some way. Report this!"));
//         }
//     }

//     match steamapi::verify_authentication_with_steam(&inner).await {
//         Ok(yeah) => match yeah {
//             true => {}
//             false => {
//                 return Ok(
//                     HttpResponse::BadRequest().body("Could not verify your identity with Steam")
//                 )
//             }
//         },
//         Err(some) => return Ok(HttpResponse::InternalServerError().body(some.to_string())),
//     }

//     let openid_identity: &String = match inner.get("openid.identity") {
//         Some(str) => str,
//         None => return Ok(HttpResponse::BadRequest().finish()),
//     };

//     // let openid_sig = inner.get("openid.sig").expect("No openid.sig on request");
//     let steamid = openid_identity.replace("https://steamcommunity.com/openid/id/", "");
//     log::info!("Openid landing received from steamid: {steamid}");
//     let client: Client = state.pool.get().await.map_err(MyError::PoolError)?;

//     let auth = match db::get_user_from_steamid(&client, &steamid).await {
//         // there is a user corresponding
//         Ok(user) => {
//             log::trace!("User found for steamid {steamid}");
//             match get_authorization_for_user(&client, &user).await {
//                 Ok(auth) => {
//                     log::debug!("Assigning {auth:?} to {user:?}");
//                     auth
//                 }
//                 Err(_) => {
//                     log::error!("Internally failed to get authorization for {user:?}");
//                     return Ok(
//                         HttpResponse::InternalServerError().body("500 Internal Server Error")
//                     );
//                 }
//             }
//         }
//         // user wasn't found
//         Err(_) => {
//             log::info!("Creating a new user with steamid {steamid}");
//             let user: User = match users::add_user_with_steamid(&state, &client, &steamid).await {
//                 Ok(user) => user,
//                 Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
//             };
//             get_authorization_for_user(&client, &user).await?
//         }
//     };
//     Ok(HttpResponse::Found()
//         .append_header((
//             "Set-Cookie",
//             format!(
//                 "auth-token={0}; Expires={1}; SameSite=Lax; Path=/",
//                 auth.token, auth.expires
//             ),
//         ))
//         .append_header((
//             "Location",
//             format!("http://{0}:80/home", state.current_host.address),
//         ))
//         .finish())
// }
