use reqwest::blocking::Client as Reqwest;
use serde::Deserialize;

use std::error::Error;

pub struct Client {
    username: String,
    password: String,
    inner: Reqwest,
}

impl Client {
    pub fn new(username: String, password: String) -> Self {
        let inner = Reqwest::new();
        Self {
            username,
            password,
            inner,
        }
    }

    pub fn fetch_surveys(&self) -> Result<Vec<Survey>, Box<dyn Error>> {
        let response = self
            .inner
            .get("https://api.surveyhero.com/v1/surveys")
            .basic_auth(&self.username, Some(&self.password))
            .send()?
            .error_for_status()?;
        let surveys: Surveys = response.json()?;
        Ok(surveys.surveys)
    }

    pub fn fetch_questions(&self, survey_id: usize) -> Result<Vec<Question>, Box<dyn Error>> {
        let response = self
            .inner
            .get(format!(
                "https://api.surveyhero.com/v1/surveys/{}/elements",
                survey_id
            ))
            .basic_auth(&self.username, Some(&self.password))
            .send()?
            .error_for_status()?;

        let elements: Elements = response.json()?;
        Ok(elements.questions().collect())
    }
}

#[derive(Debug, Deserialize)]
pub struct Elements {
    elements: Vec<Element>,
}

impl Elements {
    pub fn questions(self) -> impl Iterator<Item = Question> {
        self.elements.into_iter().filter_map(|e| {
            if let Element::Question { question, .. } = e {
                Some(question)
            } else {
                None
            }
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Element {
    #[serde(rename = "question")]
    Question {
        element_id: usize,
        question: Question,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Question {
    #[serde(rename = "choice_list")]
    ChoiceList {
        question_text: String,
        choice_list: ChoiceList,
    },
}

impl Question {
    pub fn text(&self) -> &str {
        match self {
            Self::ChoiceList { question_text, .. } => question_text,
        }
    }

    pub fn is_free_form(&self) -> bool {
        println!("TODO: free form answers");
        true
    }

    pub fn is_select_many(&self) -> bool {
        match self {
            Self::ChoiceList { choice_list, .. } => choice_list.settings.allows_multiple_choices,
        }
    }

    pub fn is_select_one(&self) -> bool {
        match self {
            Self::ChoiceList { choice_list, .. } => !choice_list.settings.allows_multiple_choices,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ChoiceList {
    choices: Vec<Choice>,
    settings: Settings,
}

impl ChoiceList {
    pub fn as_strs(&self) -> impl Iterator<Item = &str> {
        self.choices.iter().map(|c| c.label.as_str())
    }

    pub fn contains_all_answers(&self, answers: &[&str]) -> bool {
        self.as_strs().eq(answers.iter().map(|s| *s))
    }

    pub fn to_vec(&self) -> Vec<String> {
        self.as_strs().map(|s| s.to_owned()).collect()
    }
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    label: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    allows_multiple_choices: bool,
}

#[derive(Debug, Deserialize)]
pub struct Surveys {
    pub surveys: Vec<Survey>,
}

#[derive(Debug, Deserialize)]
pub struct Survey {
    pub survey_id: usize,
    pub title: String,
}