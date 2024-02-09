use std::collections::HashMap;

use common::{
    card::{Card, CardDescription, CardDescriptionPart, Id},
    card_data::CardData,
};
use rayon::iter::{FromParallelIterator, IntoParallelIterator, ParallelIterator};

use crate::{error::ProcessingError, extract::Extraction};

pub fn refine(card: Extraction) -> Result<(Vec<Id>, Card), ProcessingError> {
    let description = (&card).try_into()?;

    Ok((
        card.ids,
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

pub struct CardDataProxy(pub CardData);

impl FromParallelIterator<(Vec<Id>, Card)> for CardDataProxy {
    fn from_par_iter<T: IntoParallelIterator<Item = (Vec<Id>, Card)>>(iter: T) -> Self {
        type Entries = HashMap<Id, Card>;
        type Ids = Vec<(Id, Vec<Id>)>;

        let (entries, ids): (Entries, Ids) = iter
            .into_par_iter()
            .map(|(mut ids, card)| {
                let id = ids.remove(0);
                ((id, card), (id, ids))
            })
            .unzip();

        let alternatives = ids
            .into_iter()
            .flat_map(|(id, ids)| ids.into_iter().map(move |src| (src, id)))
            .collect();

        CardDataProxy(CardData::new(entries, alternatives))
    }
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
