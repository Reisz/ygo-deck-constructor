use common::card::{Card, CardDescription, CardDescriptionPart};

use crate::{error::ProcessingError, extract::Extraction};

pub fn refine(card: Extraction) -> Result<Card, ProcessingError> {
    let description = (&card).try_into()?;

    Ok(Card {
        name: card.name,
        ids: card.ids,
        description,
        search_text: card.description.to_lowercase(),
        card_type: card.card_type,
        limit: card.limit,
        archetype: card.archetype,
    })
}

impl TryFrom<&Extraction> for CardDescription {
    type Error = ProcessingError;

    fn try_from(card: &Extraction) -> Result<Self, Self::Error> {
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
                            card.ids.first().unwrap().get(),
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
