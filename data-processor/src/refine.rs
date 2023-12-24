use common::card::{Card, CardDescription, CardDescriptionPart, Id};

use crate::{error::ProcessingError, extract::ExtractionResult};

#[must_use]
pub fn map(card: &ExtractionResult) -> (String, Id) {
    (card.name.clone(), card.id)
}

#[must_use]
pub fn refine(card: ExtractionResult) -> Option<(Id, Card)> {
    let description = (&card).try_into().map_err(|err| eprintln!("{err}")).ok()?;
    Some((
        card.id,
        Card {
            name: card.name,
            description,
            search_text: card.description.to_lowercase(),
            card_type: card.card_type,
            limit: card.limit,
            archetype: card.archetype,
        },
    ))
}

impl TryFrom<&ExtractionResult> for CardDescription {
    type Error = ProcessingError;

    fn try_from(card: &ExtractionResult) -> Result<Self, Self::Error> {
        let mut spell_effect = None;
        let mut monster_effect = Vec::new();
        let mut current_list = None;

        for paragraph in card.description.lines() {
            if let Some(paragraph) = paragraph.strip_prefix('●') {
                current_list
                    .get_or_insert(Vec::default())
                    .push(paragraph.to_owned());
                continue;
            }

            if let Some(list) = current_list.take() {
                monster_effect.push(CardDescriptionPart::List(list));
            }

            match paragraph.trim() {
                "[ Pendulum Effect ]" => {
                    if !card.card_type.is_pendulum_monster() {
                        return Err(ProcessingError::new_unexpected(
                            card.id.get(),
                            "description",
                            "pendulum header on non-pendulum card",
                        ));
                    }

                    continue;
                }
                "[ Monster Effect ]" => {
                    spell_effect = Some(monster_effect.split_off(0));
                    continue;
                }
                _ => {}
            }

            monster_effect.push(CardDescriptionPart::Paragraph(paragraph.to_owned()));
        }

        Ok(if let Some(spell_effect) = spell_effect {
            CardDescription::Pendulum {
                spell_effect,
                monster_effect,
            }
        } else {
            CardDescription::Regular(monster_effect)
        })
    }
}
