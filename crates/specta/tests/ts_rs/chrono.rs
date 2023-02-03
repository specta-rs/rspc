#![allow(deprecated)]

use chrono::{
    Date, DateTime, Duration, FixedOffset, Local, NaiveDate, NaiveDateTime, NaiveTime, Utc,
};
use specta::Type;

use crate::ts::assert_ts;

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

    assert_ts!(
        Chrono,
        "{ date: [string, string, string, string]; time: string; date_time: [string, string, string, string]; duration: string }"
    )
}
