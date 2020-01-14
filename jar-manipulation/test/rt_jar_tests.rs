extern crate ntest_timeout;
extern crate timebomb;


#[test]
#[timeout(10000)]
pub fn can_open_rt_jar() {
    "/homes/fpn17/Desktop/jdk8u232-b09/jre/lib/rt.jar"
}
