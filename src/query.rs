pub async fn worker(
    client: &q::Q,
    semester: &str,
    course_no: &str,
    language: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let _details = client.query(semester, course_no, language).await?;

    // println!(
    //     "{: <10} | {:　<10} | {}/{}",
    //     details.course_no, details.course_name, details.choose_student, details.restrict2
    // );

    Ok(())
}
