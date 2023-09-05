use crate::tables::{
    time_points::{
        insert::insert,
        select::select,
        delete::delete,
    },
    timelines,
};

use anyhow::{ensure, Result, bail};
use common::{MPQ, consts::TEST_USER_USER, session::gen_jwt};

use color_print::cprintln;
use quickcheck::{Gen, Arbitrary};

const NUM_TESTS: usize = 1000;

pub async fn verify_insert_then_delete_time_point(
    client: &reqwest::Client,
    g: &mut Gen,
) -> Result<()> {
    print!("Verify inserting a time point then deleting it... ");

    let jwt = gen_jwt(TEST_USER_USER);

    let timeline = timelines::insert::insert(&jwt, client).await?;

    for i in 0..NUM_TESTS {
        // println!("Using JWT: {:?}", jwt);
        let value = MPQ::arbitrary(g);

        match insert(&jwt, client, value, timeline).await {
            Ok(id) => {
                match select(client, id).await {
                    Ok(time_point) => {
                        ensure!(time_point.id == id, "Test case didn't return the right time_point\nIteration: {i}\nTime_Point: {time_point:?}\nExpected id: {id}\nExpected author: {TEST_USER_USER}");
                        if let Err(e) = delete(&jwt, client, id).await {
                            bail!("Test case returned an error: {e:?}\nIteration: {i}");
                        }
                    }
                    Err(e) => bail!("Test case returned an error: {e:?}\nIteration: {i}"),
                }
            }
            Err(e) => bail!("Test case returned an error: {e:?}\nIteration: {i}"),
        }
    }

    cprintln!("<green>Success</green>");

    Ok(())
}
