use crate::{
    consts::TEST_USER_USER,
    session::gen_jwt,
    tables::timelines::{insert::insert_timeline, select::select},
};
use color_print::cprintln;

const NUM_TESTS: usize = 100;

pub async fn verify_select_timeline_after_insert(
    client: &reqwest::Client,
) -> Result<(), String> {
    print!("Verify selecting a timeline after insertion... ");

    let jwt = gen_jwt(TEST_USER_USER);

    for i in 0..NUM_TESTS {
        // println!("Using JWT: {:?}", jwt);

        match insert_timeline(&jwt, client).await {
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
