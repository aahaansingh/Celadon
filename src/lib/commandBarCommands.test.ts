import { describe, it, expect } from 'vitest';
import { isSoloListCommand, SOLO_LIST_COMMANDS } from './commandBarCommands';

describe('commandBarCommands', () => {
	describe('isSoloListCommand', () => {
		it('returns true for \\f, \\s, \\t (list views)', () => {
			expect(isSoloListCommand('\\f')).toBe(true);
			expect(isSoloListCommand('\\s')).toBe(true);
			expect(isSoloListCommand('\\t')).toBe(true);
		});

		it('returns true for \\a (filter only, so Enter does not apply article search)', () => {
			expect(isSoloListCommand('\\a')).toBe(true);
		});

		it('returns true for \\fs and \\sf (new commands)', () => {
			expect(isSoloListCommand('\\fs')).toBe(true);
			expect(isSoloListCommand('\\sf')).toBe(true);
		});

		it('returns false for commands with a query (so first suggestion is used on Enter)', () => {
			expect(isSoloListCommand('\\f:foo')).toBe(false);
			expect(isSoloListCommand('\\s:bar')).toBe(false);
			expect(isSoloListCommand('\\t:baz')).toBe(false);
			expect(isSoloListCommand('\\fs:qux')).toBe(false);
			expect(isSoloListCommand('\\sf:quux')).toBe(false);
		});

		it('returns false for plain search (article search applies)', () => {
			expect(isSoloListCommand('hello')).toBe(false);
			expect(isSoloListCommand('\\a:query')).toBe(false);
		});
	});

	describe('SOLO_LIST_COMMANDS', () => {
		it('includes all expected solo commands', () => {
			expect(SOLO_LIST_COMMANDS).toContain('\\f');
			expect(SOLO_LIST_COMMANDS).toContain('\\s');
			expect(SOLO_LIST_COMMANDS).toContain('\\t');
			expect(SOLO_LIST_COMMANDS).toContain('\\a');
			expect(SOLO_LIST_COMMANDS).toContain('\\fs');
			expect(SOLO_LIST_COMMANDS).toContain('\\sf');
			expect(SOLO_LIST_COMMANDS).toHaveLength(6);
		});
	});
});
