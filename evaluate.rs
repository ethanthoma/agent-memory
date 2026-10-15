use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::ExitCode;

mod search;

struct Task {
    category: String,
    expected_id: Option<usize>,
    query: String,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Evaluation: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<String> = env::args().skip(1).take(4).collect();
    if arguments.is_empty() || arguments.len() > 3 {
        return Err("usage: agent-memory-eval <fixture_directory> [vectors.tsv] [--details] | --texts <fixture_directory>".into());
    }
    let texts_only = arguments[0] == "--texts";
    let directory = if texts_only {
        if arguments.len() != 2 {
            return Err("--texts requires a fixture directory".into());
        }
        &arguments[1]
    } else {
        &arguments[0]
    };
    let directory = Path::new(directory);
    let mut plain = Vec::new();
    let mut cued = Vec::new();
    for line in read(&directory.join("notes.tsv"))?.lines() {
        let columns: Vec<&str> = line.split('\t').collect();
        if columns.len() != 3 {
            return Err("notes require fact, cue, and source columns".into());
        }
        plain.push(format!("{}; source: {}", columns[0], columns[2]));
        cued.push(format!(
            "{}; when: {}; source: {}",
            columns[0], columns[1], columns[2]
        ));
    }
    if plain.is_empty() || plain.len() > 128 || cued.iter().any(|note| note.len() > 512) {
        return Err(
            "fixtures require 1..128 notes of at most 512 bytes including cues and sources".into(),
        );
    }
    let mut tasks = Vec::new();
    for line in read(&directory.join("tasks.tsv"))?.lines() {
        let columns: Vec<&str> = line.split('\t').collect();
        if columns.len() != 3 || columns[2].is_empty() || columns[2].len() > 512 {
            return Err("tasks require category, expected ID or -, and query columns".into());
        }
        let expected_id = if columns[1] == "-" {
            None
        } else {
            Some(columns[1].parse::<usize>()?)
        };
        if !matches!(
            columns[0],
            "direct" | "indirect" | "associative" | "unknown"
        ) || (columns[0] == "unknown") != expected_id.is_none()
        {
            return Err("task category and expected ID disagree".into());
        }
        if expected_id.is_some_and(|id| id >= plain.len()) {
            return Err("task refers to a missing note".into());
        }
        tasks.push(Task {
            category: columns[0].to_owned(),
            expected_id,
            query: columns[2].to_owned(),
        });
    }
    if tasks.is_empty() || tasks.len() > 128 {
        return Err("fixtures require 1..128 tasks".into());
    }
    if texts_only {
        for text in plain
            .iter()
            .chain(&cued)
            .chain(tasks.iter().map(|task| &task.query))
        {
            println!("{text}");
        }
        return Ok(());
    }
    let vector_path = arguments
        .iter()
        .skip(1)
        .find(|argument| argument.as_str() != "--details");
    let vectors = match vector_path {
        Some(path) => {
            let mut vectors = Vec::new();
            let texts: Vec<&str> = plain
                .iter()
                .chain(&cued)
                .chain(tasks.iter().map(|task| &task.query))
                .map(String::as_str)
                .collect();
            for (index, line) in read(Path::new(path))?.lines().enumerate() {
                let (text, values) = line
                    .split_once('\t')
                    .ok_or("embedding row requires text and vector columns")?;
                if texts.get(index).copied() != Some(text) {
                    return Err("embedding text differs from fixture; regenerate vectors".into());
                }
                let vector: Vec<f64> = values
                    .split('\t')
                    .map(str::parse)
                    .collect::<Result<_, _>>()?;
                if vector.is_empty()
                    || vector.len() > 4096
                    || vector.iter().any(|value| !value.is_finite())
                {
                    return Err("invalid embedding vector".into());
                }
                let norm = vector.iter().map(|value| value * value).sum::<f64>().sqrt();
                if !norm.is_finite() || norm <= 0.0 {
                    return Err("embedding must have a finite nonzero norm".into());
                }
                vectors.push(
                    vector
                        .into_iter()
                        .map(|value| value / norm)
                        .collect::<Vec<_>>(),
                );
            }
            if vectors.len() != plain.len() * 2 + tasks.len()
                || vectors
                    .iter()
                    .any(|vector| vector.len() != vectors[0].len())
            {
                return Err("embedding row count or dimensions do not match --texts output".into());
            }
            Some(vectors)
        }
        None => None,
    };
    let details = arguments.iter().any(|argument| argument == "--details");
    if details {
        println!("method\ttask_index\tcategory\texpected_id\treturned_ids\tcontext_bytes");
    } else {
        println!(
            "| Method | Direct hit | Indirect hit | Associative hit | MRR@3 | Unknown abstention | Mean context bytes |"
        );
        println!("| --- | --- | --- | --- | --- | --- | --- |");
    }
    for (corpus_name, notes, offset) in [("plain", &plain, 0), ("cues", &cued, plain.len())] {
        for method in ["substring", "BM25", "MiniLM"] {
            if method == "MiniLM" && vectors.is_none() {
                continue;
            }
            let mut hits = [0_u32; 3];
            let mut totals = [0_u32; 3];
            let mut unknown_total = 0_u32;
            let mut abstentions = 0_u32;
            let mut reciprocal_rank_total = 0.0;
            let mut bytes_total = 0_usize;
            for (task_index, task) in tasks.iter().enumerate() {
                let ranking = match method {
                    "substring" => {
                        let query = task.query.to_lowercase();
                        notes
                            .iter()
                            .enumerate()
                            .rev()
                            .filter(|(_, note)| note.to_lowercase().contains(&query))
                            .map(|(id, _)| id)
                            .collect()
                    }
                    "BM25" => search::rank(notes, &task.query),
                    _ => {
                        let vectors = vectors.as_ref().ok_or("missing vectors")?;
                        let query = &vectors[plain.len() * 2 + task_index];
                        let mut scored: Vec<(usize, f64)> = (0..notes.len())
                            .map(|id| {
                                let score = vectors[offset + id]
                                    .iter()
                                    .zip(query)
                                    .map(|(left, right)| left * right)
                                    .sum();
                                (id, score)
                            })
                            .collect();
                        scored.sort_by(|(left_id, left), (right_id, right)| {
                            right.total_cmp(left).then_with(|| right_id.cmp(left_id))
                        });
                        scored.into_iter().map(|(id, _)| id).collect()
                    }
                };
                let selected = search::within_budget(notes, &ranking);
                let bytes: usize = selected
                    .iter()
                    .map(|id| format!("#{id} {}\n", notes[*id]).len())
                    .sum();
                bytes_total += bytes;
                if let Some(expected_id) = task.expected_id {
                    let category_index = match task.category.as_str() {
                        "direct" => 0,
                        "indirect" => 1,
                        "associative" => 2,
                        _ => return Err("unknown task category".into()),
                    };
                    totals[category_index] += 1;
                    if let Some(index) = selected.iter().position(|id| *id == expected_id) {
                        hits[category_index] += 1;
                        reciprocal_rank_total += 1.0 / (index + 1) as f64;
                    }
                } else {
                    unknown_total += 1;
                    if selected.is_empty() {
                        abstentions += 1;
                    }
                }
                if details {
                    let expected = task
                        .expected_id
                        .map_or_else(|| "-".to_owned(), |id| id.to_string());
                    let returned = selected
                        .iter()
                        .map(usize::to_string)
                        .collect::<Vec<_>>()
                        .join(",");
                    println!(
                        "{method}-{corpus_name}\t{task_index}\t{}\t{expected}\t{returned}\t{bytes}",
                        task.category
                    );
                }
            }
            if !details {
                let positive_count: u32 = totals.iter().sum();
                if positive_count == 0 {
                    return Err("evaluation requires positive tasks".into());
                }
                println!(
                    "| {method}-{corpus_name} | {}/{} | {}/{} | {}/{} | {:.3} | {abstentions}/{unknown_total} | {} |",
                    hits[0],
                    totals[0],
                    hits[1],
                    totals[1],
                    hits[2],
                    totals[2],
                    reciprocal_rank_total / f64::from(positive_count),
                    bytes_total / tasks.len()
                );
            }
        }
    }
    Ok(())
}

fn read(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let mut text = String::new();
    File::open(path)?
        .take(8 * 1024 * 1024 + 1)
        .read_to_string(&mut text)?;
    if text.len() > 8 * 1024 * 1024 {
        return Err("evaluation input exceeds 8 MiB".into());
    }
    Ok(text)
}
