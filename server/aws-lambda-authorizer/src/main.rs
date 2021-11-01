use legion_aws::lambda::lambda;

mod handler;

use handler::handler;

fn main() -> anyhow::Result<()> {
    simple_logger::init_with_level(log::Level::Info)?;

    lambda!(handler)
}
