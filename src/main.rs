use scraper::{Html, Selector};
use std::fmt::Debug;

trait Trending: Debug + Default {
    fn parse_html(content: String) -> Vec<Self>
    where
        Self: Sized;
}

#[derive(Debug, Default)]
pub struct Collaborator {
    name: String,
    avatar: String,
}

#[derive(Debug, Default)]
pub struct Repository {
    author: String,
    name: String,
    link: String,
    description: String,
    star_count: u32,
    add: String,
    forks: u32,
    language: String,
    build_by: Vec<Collaborator>,
}

#[allow(dead_code)]
pub struct Developer {
    name: String,
    username: String,
    popular_repo: String,
    description: String,
}

impl Trending for Repository {
    fn parse_html(content: String) -> Vec<Self> {
        let doucument = Html::parse_document(&content);
        let p_selector = Selector::parse(r#"p"#).unwrap();
        let a_selector = Selector::parse(r#"a"#).unwrap();
        let img_selector = Selector::parse(r#"img"#).unwrap();
        let div_selector = Selector::parse(r#"div"#).unwrap();
        let span_selector = Selector::parse(r#"span"#).unwrap();
        let h2_selector = Selector::parse(r#"h2[class="h3 lh-condensed"]"#).unwrap();
        let article_selector = Selector::parse(r#"article[class="Box-row"]"#).unwrap();
        let program_span_sel = Selector::parse(r#"span[itemprop="programmingLanguage"]"#).unwrap();

        let mut repos: Vec<Repository> = Vec::new();
        let mut url: String = "https://github.com".to_string();

        for per_repo in doucument.select(&article_selector) {
            let mut repo = Self::default();
            assert_eq!(per_repo.value().name(), "article");
            if let Some(p) = per_repo.select(&p_selector).next() {
                repo.description = p
                    .text()
                    .collect::<Vec<_>>()
                    .into_iter()
                    .map(|x| x.to_string().trim().to_string())
                    .collect();
            }

            let a_link = per_repo
                .select(&h2_selector)
                .next()
                .unwrap()
                .select(&a_selector)
                .next()
                .unwrap();

            let repo_link = a_link.value().attr("href").unwrap();
            url.push_str(repo_link);
            repo.link = url.clone();
            url = url.replace(repo_link, "");

            let tmp = a_link
                .text()
                .collect::<Vec<_>>()
                .into_iter()
                .map(|e| e.to_string().trim().to_owned())
                .collect::<String>()
                .to_owned();

            let name_vec = tmp.split(' ').collect::<Vec<_>>();
            repo.author = name_vec[0].to_string();
            repo.name = name_vec[1].to_string().replace('/', "");

            let div = per_repo.select(&div_selector).nth(2).unwrap();
            if let Some(span) = div.select(&program_span_sel).next() {
                repo.language = span.text().collect();
            }

            let mut repo_link = repo_link.to_owned();
            repo_link.push_str("/stargazers");

            let mut attr = "a[href=\"".to_string();
            attr.push_str(&repo_link);
            attr.push_str("\"]");

            let start_a_sel = Selector::parse(&attr).unwrap();
            let star_count = div
                .select(&start_a_sel)
                .next()
                .unwrap()
                .text()
                .collect::<String>();
            repo.star_count = star_count
                .replace(',', "")
                .trim()
                .split(' ')
                .next()
                .unwrap()
                .parse()
                .unwrap();
            attr = attr.replace("/stargazers", "/forks");
            let fork_a_sel = Selector::parse(&attr).unwrap();
            let fork_count = div
                .select(&fork_a_sel)
                .next()
                .unwrap()
                .text()
                .collect::<String>();
            repo.forks = fork_count.replace(',', "").trim().parse().unwrap();

            let mut spans: Vec<_> = per_repo
                .select(&span_selector)
                .filter(|span| span.value().attr("itemprop").is_none())
                .collect();

            spans.reverse();
            repo.add = spans
                .first()
                .unwrap()
                .text()
                .collect::<String>()
                .trim()
                .to_string();

            let mut collaborators = vec![];

            for col_a_link in spans.get(1).unwrap().select(&a_selector) {
                let mut collaborator = Collaborator::default();
                let col_avator_img = col_a_link.select(&img_selector).next().unwrap();
                collaborator.name = col_avator_img
                    .value()
                    .attr("alt")
                    .unwrap()
                    .to_string()
                    .split('@')
                    .collect();

                collaborator.avatar = col_avator_img.value().attr("src").unwrap().to_string();

                collaborators.push(collaborator);
            }
            repo.build_by = collaborators;

            repos.push(repo);
        }

        repos
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proxy = reqwest::Proxy::all("http://127.0.0.1:7897")?;
    let client = reqwest::Client::builder().proxy(proxy).build()?;
    let res = client.get("https://github.com/trending").send().await?;
    assert!(res.status().is_success());

    let text = res.text().await?;
    let repos = Repository::parse_html(text);
    dbg!(repos);

    Ok(())
}
