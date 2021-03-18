use std::path::Path;
use common::task::TaskResult;
use common::case::{CaseResult, CaseState};
use common::error::Error;

pub async fn report<P: AsRef<Path>>(task_result: &TaskResult, path: P) -> Result<(), Error> {
    let rwr = csv::Writer::from_path(path);
    let mut rwr = match rwr{
        Ok(w) => w,
        Err(_) => return Err(Error::new("000", "path error"))
    };

    let empty = &vec![];
    let cr_vec = match task_result {
        Ok(tr) => tr.result(),
        Err(_) => empty
    };

    if cr_vec.len() == 0 {
        return Ok(());
    }
    let head_vec = to_head_vec(cr_vec);

    let _ = rwr.write_record(&head_vec);
    cr_vec.iter()
        .map(|(_, cr)| to_value_vec(cr, head_vec.len()))
        .for_each(|sv| rwr.write_record(&sv).unwrap());

    rwr.flush()?;
    return Ok(());
}



fn to_value_vec(cr: &CaseResult, head_len: usize) -> Vec<String> {

    let empty = &vec![];
    let pr_vec = match cr {
        Ok(cr) => cr.result(),
        Err(_) =>  empty
    };

    let mut vec: Vec<String> = pr_vec.iter()
        .map(|(_, pr)| match pr {
            Ok(_) => String::from("O"),
            Err(_) => String::from("X")
        })
        .collect();

    if vec.len() < head_len -3 {
        for _i in 0..head_len -3 - vec.len() {
            vec.push(String::from(""));
        }
    }

    match cr {
        Ok(crs) => {
            match crs.state() {
                CaseState::Ok => {
                    vec.push(String::from("O"));
                    vec.push(String::from(""));
                    vec.push(String::from(""));
                },
                CaseState::PointError(e) => {
                    vec.push(String::from("X"));
                    vec.push(String::from(""));
                    vec.push(String::from(format!("{}", e)));
                },
                CaseState::PointFailure => {

                    vec.push(String::from("X"));
                    vec.push(String::from(""));
                    let  (_, pr) = pr_vec.last().unwrap();
                    match pr {
                        Ok(pr) => {
                            vec.push(String::from(format!("{}",pr.result())));
                        },
                        Err(e) => {
                            vec.push(String::from(format!("{}", e)));
                        }
                    }

                }
            }


        },
        Err(e) => {
            vec.push(String::from("X"));
            vec.push(format!("{}", e));
            vec.push(String::from(""));
        }
    };
    vec
}

fn to_head_vec(cr_vec: &Vec<(usize, CaseResult)>) -> Vec<String> {

    let (_, max_len_cr) = cr_vec.iter().max_by(
        |(_, x), (_, y)| {
            let x = match x {
                Ok(pv) => pv.result().len(),
                Err(_) => 0
            };
            let y = match y {
                Ok(pv) => pv.result().len(),
                Err(_) => 0
            };
            x.cmp(&y)
        })
    .unwrap();

    let empty = &vec![];
    let pr_vec =  match max_len_cr {
        Ok(cr) => cr.result(),
        Err(_) => empty
    };

    let mut vec: Vec<String> = pr_vec.iter()
        .map(|(pid, _)| pid)
        .map(|pid| String::from(pid))
        .collect();
    vec.push(String::from("caseResult"));
    vec.push(String::from("caseInfo"));
    vec.push(String::from("lastPointInfo"));
    vec
}
