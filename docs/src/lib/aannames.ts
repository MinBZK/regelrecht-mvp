/*
 * Aannames: the scales, the ordering and the integrity checks behind
 * /roadmap/aannames.
 *
 * The page works the "riskiest assumption first" method: every aanname
 * gets two scores, impact (how much rests on it) and zekerheid (how much
 * evidence there is for it), and the ones that are important and least
 * supported get tested first. One file per aanname under
 * src/content/roadmap/aannames/, with frontmatter only, like the werkpakketten.
 *
 * An aanname points at werkpakketten, not the other way round. The werkpakket
 * page derives its own list from these pointers, so the relation is written
 * down once and cannot drift between two files.
 */

/*
 * Both scales run 1 to 5, and every level has a fixed description. A score is
 * a judgement, and the description is what makes it one you can argue with:
 * "zekerheid 3" means a verkenning or prototype supports it, not "fairly
 * sure". A continuous 0–100 scale was considered and dropped, because nobody
 * can defend the difference between 62 and 70.
 *
 * Aannames with the same two scores share a cell on the grid. That is a
 * feature: two aannames in one cell are, as far as the method is concerned,
 * equally urgent.
 */
export const IMPACT_NIVEAUS = [
  {
    score: 1,
    label: 'Marginaal',
    omschrijving: 'Klopt het niet, dan verandert er vrijwel niets aan het plan.',
  },
  {
    score: 2,
    label: 'Beperkt',
    omschrijving: 'Klopt het niet, dan stuurt één werkpakket bij.',
  },
  {
    score: 3,
    label: 'Merkbaar',
    omschrijving:
      'Klopt het niet, dan moeten meerdere werkpakketten hun aanpak herzien.',
  },
  {
    score: 4,
    label: 'Groot',
    omschrijving:
      'Klopt het niet, dan moet een fase of een hele discipline anders.',
  },
  {
    score: 5,
    label: 'Fundamenteel',
    omschrijving:
      'Klopt het niet, dan staat de aanpak van RegelRecht zelf ter discussie.',
  },
] as const;

export const ZEKERHEID_NIVEAUS = [
  {
    score: 1,
    label: 'Onderbuikgevoel',
    omschrijving: 'Er is niet naar gekeken; het is een overtuiging.',
  },
  {
    score: 2,
    label: 'Aanwijzingen',
    omschrijving:
      'Losse gesprekken, anekdotes of een vergelijking met elders wijzen die kant op.',
  },
  {
    score: 3,
    label: 'Eerste bewijs',
    omschrijving:
      'Een verkenning, prototype of reeks interviews ondersteunt het.',
  },
  {
    score: 4,
    label: 'Sterk bewijs',
    omschrijving:
      'Herhaald getoetst, bijvoorbeeld in een referentiecasus, en het hield stand.',
  },
  {
    score: 5,
    label: 'Bewezen',
    omschrijving: 'In de praktijk aangetoond en vastgelegd.',
  },
] as const;

type NonEmpty = [number, ...number[]];
export const SCORES = IMPACT_NIVEAUS.map((n) => n.score) as NonEmpty;
export const SCORE_MIN = SCORES[0];
export const SCORE_MAX = SCORES[SCORES.length - 1];

export const getImpact = (score: number) =>
  IMPACT_NIVEAUS.find((n) => n.score === score);
export const getZekerheid = (score: number) =>
  ZEKERHEID_NIVEAUS.find((n) => n.score === score);

/*
 * The zone the method is named after: important and barely supported. On the
 * grid it is the tinted corner top left, and the ordered list marks it too.
 * The thresholds are the outer two levels of each scale, so the middle row and
 * column belong to neither side; an aanname scored 3 is undecided, and the
 * page should not pretend otherwise.
 */
export const eerstToetsen = (impact: number, zekerheid: number) =>
  impact >= 4 && zekerheid <= 2;

/*
 * Two kinds of aanname, taken over from the pre-mortem the first set came
 * from. A hypothese is particular to RegelRecht, so there is something to
 * learn and it gets tested. A randvoorwaarde holds for any project (funding, a
 * stable team) and is watched rather than tested. Both go on the same grid,
 * because the method asks how much rests on an aanname and how much evidence
 * there is, whatever kind it is.
 */
export const SOORTEN = [
  {
    id: 'hypothese',
    label: 'Hypothese',
    tagColor: 'accent',
    omschrijving:
      'Specifiek voor RegelRecht. Hier valt iets te leren, dus toetsen we ze.',
  },
  {
    id: 'randvoorwaarde',
    label: 'Randvoorwaarde',
    tagColor: 'neutral',
    omschrijving:
      'Geldt voor elk project. Die bewaken we in plaats van ze te toetsen.',
  },
] as const;
export const SOORT_IDS = SOORTEN.map((s) => s.id) as [string, ...string[]];
export const getSoort = (id: string) => SOORTEN.find((s) => s.id === id);

export interface AannameData {
  id: string;
  titel: string;
  aanname: string;
  soort: string;
  // Both or neither: an aanname can be written down before anyone has placed
  // it, and then it waits under the grid instead of getting a made-up score.
  impact?: number;
  zekerheid?: number;
  onderbouwing: string;
  toetsing: string;
  // The pre-mortem clusters behind the aanname: groups of cards that named
  // the same way of failing, with how many cards each.
  clusters: { naam: string; kaartjes: number }[];
  werkpakketIds: string[];
}

export type Gescoord = AannameData & { impact: number; zekerheid: number };
export const gescoord = (a: AannameData): a is Gescoord =>
  a.impact !== undefined && a.zekerheid !== undefined;

/*
 * Risk is impact times the lack of evidence: 5 × (6 − 1) = 25 for the worst
 * case, 1 × (6 − 5) = 1 for the least. A tie goes to the higher impact, so of
 * two aannames with the same product the one with more resting on it comes
 * first, and then to the titel, so the order is stable between builds.
 */
export const risico = (a: { impact: number; zekerheid: number }) =>
  a.impact * (SCORE_MAX + 1 - a.zekerheid);

/** The scored aannames, riskiest first. Unscored ones have no place here. */
export function opRisico<T extends { data: AannameData }>(
  alle: T[],
): (T & { data: Gescoord })[] {
  return (
    alle.filter((a) => gescoord(a.data)) as (T & { data: Gescoord })[]
  ).sort(
    (a, b) =>
      risico(b.data) - risico(a.data) ||
      b.data.impact - a.data.impact ||
      a.data.titel.localeCompare(b.data.titel, 'nl'),
  );
}

/**
 * Every aanname in reading order: the scored ones riskiest first, then the
 * ones still waiting for a place, by titel. For lists that show both, such as
 * the aannames under a werkpakket.
 */
export function inLeesvolgorde<T extends { data: AannameData }>(alle: T[]): T[] {
  return [...opRisico(alle), ...ongescoord(alle)];
}

/** The aannames without a score yet, by titel. */
export function ongescoord<T extends { data: AannameData }>(alle: T[]): T[] {
  return alle
    .filter((a) => !gescoord(a.data))
    .sort((a, b) => a.data.titel.localeCompare(b.data.titel, 'nl'));
}

export function aannamesInCel<T extends { data: AannameData }>(
  alle: T[],
  impact: number,
  zekerheid: number,
): T[] {
  return opRisico(
    alle.filter(
      (a) => a.data.impact === impact && a.data.zekerheid === zekerheid,
    ),
  );
}

/**
 * Fails the build on an aanname that points at a werkpakket that does not
 * exist, names the same werkpakket twice, or sits in a file whose name is not
 * its id. The same rules the werkpakketten follow, for the same reasons: the
 * id is what a link from a werkpakket page carries, so it has to be unique and
 * findable with `ls`.
 */
export function assertAannameReferences(
  aannames: { data: AannameData; id?: string }[],
  werkpakketIds: Set<string>,
): void {
  const problems: string[] = [];
  const seen = new Set<string>();
  for (const { data, id: bestandsnaam } of aannames) {
    const waar = `aanname ${data.id} (${data.titel})`;
    if (seen.has(data.id)) {
      problems.push(`id "${data.id}" wordt door meer dan één bestand gebruikt`);
    }
    seen.add(data.id);
    if (bestandsnaam !== undefined && bestandsnaam !== data.id) {
      problems.push(
        `bestand "${bestandsnaam}.md" bevat id "${data.id}"; die horen gelijk te zijn`,
      );
    }
    const genoemd = new Set<string>();
    for (const wpId of data.werkpakketIds) {
      if (!werkpakketIds.has(wpId)) {
        problems.push(`${waar}: werkpakket "${wpId}" bestaat niet`);
      }
      if (genoemd.has(wpId)) {
        problems.push(`${waar}: werkpakket "${wpId}" staat er twee keer in`);
      }
      genoemd.add(wpId);
    }
  }
  if (problems.length) {
    throw new Error(
      `Aanname-verwijzingen kloppen niet:\n  ${problems.join('\n  ')}`,
    );
  }
}

/** "Impact 5 · Fundamenteel, zekerheid 1 · Onderbuikgevoel" */
export const scoreTekst = (a: AannameData) =>
  gescoord(a)
    ? `Impact ${a.impact} · ${getImpact(a.impact)?.label}, ` +
      `zekerheid ${a.zekerheid} · ${getZekerheid(a.zekerheid)?.label}`
    : 'Nog niet gescoord';

export const AANNAMES_PAD = '/roadmap/aannames';
export const aannameHref = (id: string) => `${AANNAMES_PAD}#${id}`;
