use crate::tables::{
    time_points::{
        insert::insert,
        select::select,
        delete::delete,
    },
    timelines,
};

use anyhow::{ensure, Result, Context};
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

    let timeline = timelines::insert::insert(&jwt, client)
        .await
        .context("Couldn't insert timeline")?;

    for i in 0..NUM_TESTS {
        // println!("Using JWT: {:?}", jwt);
        let value = MPQ::arbitrary(g);

        let id = insert(&jwt, client, value, timeline)
            .await
            .context("Couldn't insert time point\nIteration: {i}")?;

        let time_point = select(client, id)
            .await
            .context("Couldn't select time point\nIteration: {i}")?;

        ensure!(
            time_point.id == id,
            "Test case didn't return the right time_point\nIteration: {i}\nTime_Point: {time_point:?}\nExpected id: {id}\nExpected author: {TEST_USER_USER}"
        );

        delete(&jwt, client, id)
            .await
            .context("Couldn't delete time point\nIteration: {i}")?;
    }

    cprintln!("<green>Success</green>");

    Ok(())
}
