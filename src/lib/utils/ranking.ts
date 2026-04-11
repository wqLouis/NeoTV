import type { SpeedTestResult } from './speedTest';
import type { SearchResult } from '$lib/api/search';

export interface ScoredSource {
	result: SearchResult;
	score: number;
	speedMs?: number;
	tagMatchBonus: number;
}

export interface SearchGroup {
	id: string;
	name: string;
	cover?: string;
	year?: string;
	typeName?: string;
	tags: string[];
	sources: ScoredSource[];
	totalScore: number;
	isMovie: boolean;
}

function normalizeName(name: string): string {
	return name.toLowerCase().replace(/\s+/g, '').trim();
}

function extractTags(result: SearchResult): string[] {
	const tags: string[] = [];
	if (result.type_name) {
		const typeParts = result.type_name.split(/[，,、\/]/);
		for (const part of typeParts) {
			const trimmed = part.trim();
			if (trimmed) tags.push(trimmed);
		}
	}
	return tags;
}

function calculateTitleScore(query: string, title: string): number {
	const q = query.toLowerCase().trim();
	const t = title.toLowerCase().trim();

	if (t === q) return 40;
	if (t.includes(q)) return 30;

	const qWords = q.split(/\s+/).filter((w) => w.length > 1);
	const matchCount = qWords.filter((w) => t.includes(w)).length;
	if (matchCount > 0) {
		return 10 + Math.min(matchCount * 5, 10);
	}

	const qChars = q.replace(/\s/g, '');
	const tChars = t.replace(/\s/g, '');
	let matches = 0;
	for (const char of qChars) {
		if (tChars.includes(char)) matches++;
	}
	if (matches > qChars.length * 0.7) return 15;

	return 5;
}

function calculateSpeedScore(latencyMs: number | undefined): number {
	if (latencyMs === undefined) return 0;
	if (latencyMs < 500) return 30;
	if (latencyMs < 1000) return 25;
	if (latencyMs < 2000) return 20;
	if (latencyMs < 3000) return 15;
	if (latencyMs < 5000) return 10;
	return 5;
}

function calculateTypeBonus(resultType: string, isMovieSearch: boolean): number {
	const type = resultType.toLowerCase();
	const isMovie = type.includes('电影') || type.includes('movie');

	if (!isMovieSearch) return 0;
	return isMovie ? 10 : -5;
}

function calculateTagMatchBonus(resultTags: string[], searchTags: string[]): number {
	if (searchTags.length === 0) return 0;

	const resultTagSet = new Set(resultTags.map((t) => t.toLowerCase()));
	const searchTagSet = new Set(searchTags.map((t) => t.toLowerCase()));

	let matches = 0;
	for (const tag of searchTags) {
		if (resultTagSet.has(tag.toLowerCase())) {
			matches++;
		}
	}

	const overlapRatio = matches / Math.max(searchTags.length, resultTags.length);
	return Math.round(overlapRatio * 10);
}

export function calculateRelevanceScore(
	result: SearchResult,
	userQuery: string,
	speedResult?: SpeedTestResult,
	isMovieSearch = false,
	searchTags: string[] = []
): { score: number; tagMatchBonus: number } {
	const titleScore = calculateTitleScore(userQuery, result.vod_name);
	const speedScore = calculateSpeedScore(speedResult?.latency_ms);
	const typeBonus = calculateTypeBonus(result.type_name, isMovieSearch);
	const resultTags = extractTags(result);
	const tagMatchBonus = calculateTagMatchBonus(resultTags, searchTags);

	const totalScore = titleScore + speedScore + typeBonus + tagMatchBonus;
	return { score: totalScore, tagMatchBonus };
}

export interface RawGroup {
	key: string;
	normalizedName: string;
	tags: string[];
	sources: SearchResult[];
}

export function groupByNameAndTags(results: SearchResult[]): RawGroup[] {
	const groupMap = new Map<string, RawGroup>();

	for (const result of results) {
		const normName = normalizeName(result.vod_name);
		const tags = extractTags(result);
		const tagKey = tags
			.map((t) => t.toLowerCase())
			.sort()
			.join('|');

		for (const existing of groupMap.values()) {
			if (existing.normalizedName === normName) {
				const existingTagSet = new Set(existing.tags.map((t) => t.toLowerCase()));
				const newTagSet = new Set(tags.map((t) => t.toLowerCase()));
				const allTagsMatch = tags.every((t) => existingTagSet.has(t.toLowerCase()));
				const hasOverlap = tags.some((t) => existingTagSet.has(t.toLowerCase()));

				if (allTagsMatch || (hasOverlap && tags.length > 0)) {
					existing.sources.push(result);
					for (const tag of tags) {
						if (!existing.tags.includes(tag)) {
							existing.tags.push(tag);
						}
					}
					break;
				}
			}
		}

		if (![...groupMap.values()].some((g) => g.normalizedName === normName)) {
			groupMap.set(normName, {
				key: normName,
				normalizedName: normName,
				tags,
				sources: [result]
			});
		}
	}

	return [...groupMap.values()];
}

export function sortGroupsByScore(
	groups: RawGroup[],
	speedCache: Map<string, SpeedTestResult>,
	userQuery: string,
	isMovieSearch = false,
	searchTags: string[] = []
): SearchGroup[] {
	const scoredGroups: SearchGroup[] = [];

	for (const group of groups) {
		const firstSource = group.sources[0];
		const scoredSources: ScoredSource[] = group.sources.map((result) => {
			const sourceKey = result.source_code;
			const speedResult = speedCache.get(sourceKey);
			const { score, tagMatchBonus } = calculateRelevanceScore(
				result,
				userQuery,
				speedResult,
				isMovieSearch,
				searchTags
			);
			return {
				result,
				score,
				speedMs: speedResult?.latency_ms,
				tagMatchBonus
			};
		});

		scoredSources.sort((a, b) => b.score - a.score);

		const avgScore = scoredSources.reduce((sum, s) => sum + s.score, 0) / scoredSources.length;
		const isMovie =
			firstSource.type_name.toLowerCase().includes('电影') ||
			firstSource.type_name.toLowerCase().includes('movie');

		scoredGroups.push({
			id: firstSource.vod_id,
			name: firstSource.vod_name,
			cover: firstSource.vod_pic,
			year: firstSource.vod_year,
			typeName: firstSource.type_name,
			tags: group.tags,
			sources: scoredSources,
			totalScore: Math.round(avgScore * scoredSources.length),
			isMovie
		});
	}

	scoredGroups.sort((a, b) => b.totalScore - a.totalScore);

	return scoredGroups;
}
