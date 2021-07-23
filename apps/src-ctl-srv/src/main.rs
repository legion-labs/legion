use legion_src_ctl::*;
use warp::Filter;

static mut G_POOL: Option<SqlConnectionPool> = None;

fn dispatch_request(pool: &SqlConnectionPool, body: bytes::Bytes) -> String {
    println!("{:?}", std::thread::current().id());
    println!("{:?}", pool);
    format!("Pong {}", std::str::from_utf8(&body).unwrap())
}

#[tokio::main]
async fn main() {
    let sql_uri = std::env::var("LEGION_SRC_CTL_DATABASE_SERVER_URI")
        .expect("missing env variable LEGION_SRC_CTL_DATABASE_SERVER_URI");
    match make_sql_connection_pool(&sql_uri) {
        Ok(new_pool) => unsafe {
            G_POOL = Some(new_pool);
        },
        Err(e) => {
            println!("{}", e);
            std::process::exit(1);
        }
    }

    let command_filter = warp::path("lsc")
        .and(warp::body::bytes())
        .map(|body: bytes::Bytes| dispatch_request(unsafe { G_POOL.as_ref().unwrap() }, body));

    warp::serve(command_filter).run(([0, 0, 0, 0], 8080)).await;
}
