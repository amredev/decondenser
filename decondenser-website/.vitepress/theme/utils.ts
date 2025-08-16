import { ContentData } from "vitepress";

export function formatDate(date: string) {
    return new Date(date).toISOString().split("T")[0];
}

export function sortedContentData(posts: ContentData[]): ContentData[] {
    return [...posts].sort((a, b) => {
        const dateA = dateFromFrontmatter(a).getTime();
        const dateB = dateFromFrontmatter(b).getTime();
        return dateB - dateA;
    });
}

function dateFromFrontmatter(post: ContentData): Date {
    if (typeof post.frontmatter.date != "string") {
        const value = JSON.stringify(post.frontmatter, null, 2);
        throw new Error(`Invalid frontmatter 'date' format: ${value}`);
    }
    return new Date(post.frontmatter.date);
}
