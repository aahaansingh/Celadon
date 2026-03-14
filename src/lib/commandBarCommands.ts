/**
 * Solo list commands: when the user presses Enter with only one of these,
 * we run parseAndExecute (list view or filter) instead of applying the first suggestion.
 * Plain article search (\a) and list commands (\f, \s, \t, \fs, \sf) must not trigger
 * "select first suggestion" behavior.
 */
export const SOLO_LIST_COMMANDS = ['\\f', '\\s', '\\t', '\\a', '\\fs', '\\sf'] as const;

export function isSoloListCommand(trimmed: string): boolean {
	return (SOLO_LIST_COMMANDS as readonly string[]).includes(trimmed);
}
