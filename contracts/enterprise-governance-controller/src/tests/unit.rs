use enterprise_protocol::error::DaoResult;

#[test]
fn initial_test() -> DaoResult<()> {
    assert_eq!(2 + 2, 4);

    Ok(())
}
