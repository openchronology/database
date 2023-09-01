use crate::tables::timelines::{
    insert::insert,
    select::select,
    delete::delete,
};
use common::{consts::TEST_USER_USER, session::gen_jwt};

use color_print::cprintln;

const NUM_TESTS: usize = 1000;

pub async fn verify_insert_then_delete_timeline(
    client: &reqwest::Client,
) -> Result<(), String> {
    print!("Verify inserting a timeline then deleting it... ");

    let jwt = gen_jwt(TEST_USER_USER);

    for i in 0..NUM_TESTS {
        // println!("Using JWT: {:?}", jwt);

        match insert(&jwt, client).await {
            Ok(id) => {
                match select(client, id).await {
                    Ok(timeline) => {
                        if !(timeline.id == id && timeline.author == TEST_USER_USER) {
                            return Err(
                                format!(
                                    "Test case didn't return the right timeline\nIteration: {:?}\nTimeline: {:?}\nExpected id: {}\nExpected author: {}",
                                    i,
                                    timeline,
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
