use chrono::{
    Date, DateTime, Duration, FixedOffset, Local, NaiveDate, NaiveDateTime, NaiveTime, Utc,
};
use specta::{ts::ts_inline, Type};

#[test]
fn chrono() {
    #[derive(Type)]
    #[allow(dead_code)]
    struct Chrono {
        date: (NaiveDate, Date<Utc>, Date<Local>, Date<FixedOffset>),
        time: NaiveTime,
        date_time: (
            NaiveDateTime,
            DateTime<Utc>,
            DateTime<Local>,
            DateTime<FixedOffset>,
        ),
        duration: Duration,
    }

    assert_eq!(
        ts_inline::<Chrono>(),
        "{ date: [string, string, string, string], time: string, date_time: [string, string, string, string], duration: string }"
    )
}
