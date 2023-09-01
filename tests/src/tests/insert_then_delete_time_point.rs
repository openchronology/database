use crate::tables::{
    time_points::{
        insert::insert,
        select::select,
        delete::delete,
    },
    timelines,
};

use common::{MPQ, consts::TEST_USER_USER, session::gen_jwt};

use color_print::cprintln;
use quickcheck::{Gen, Arbitrary};

const NUM_TESTS: usize = 1000;

pub async fn verify_insert_then_delete_time_point(
    client: &reqwest::Client,
    g: &mut Gen,
) -> Result<(), String> {
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
                        if !(time_point.id == id) {
                            return Err(
                                format!(
                                    "Test case didn't return the right time_point\nIteration: {:?}\nTime_Point: {:?}\nExpected id: {}\nExpected author: {}",
                                    i,
                                    time_point,
                                    id,
                                    TEST_USER_USER,
                                )
                            );
                        }
                        match delete(&jwt, client, id).await {
                            Ok(()) => {}
                            Err(e) => return Err(
                                format!(
                                    "Test case returned an error: {:?}\nIteration: {:?}",
                                    e,
                                    i,
                                )
                            ),
                        }
                    }
                    Err(e) => return Err(
                        format!(
                            "Test case returned an error: {:?}\nIteration: {:?}",
                            e,
                            i,
                        )
                    ),
                }
            }
            Err(e) => return Err(
                format!(
                    "Test case returned an error: {:?}\nIteration: {:?}",
                    e,
                    i,
                )
            ),
        }
    }

    cprintln!("<green>Success</green>");

    Ok(())
}
