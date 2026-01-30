import fm from 'front-matter';

interface PromptAttributes {
    name: string;
    icon: string;
    description: string;
    order?: number;
}

const glob = import.meta.glob('./contents/*.md', { eager: true, query: '?raw' });

const prompts = Object.entries(glob).map(([path, mod]) => {
    // mod: { default: string } because of ?raw query
    const content = (mod as { default: string }).default;
    const parsed = fm<PromptAttributes>(content);

    // Extract ID from filename: "./contents/my-prompt.md" -> "my_prompt"
    const filename = path.split('/').pop()?.replace(/\.md$/, '') || 'unknown';
    const id = filename.replace(/-/g, '_');

    return {
        id,
        name: parsed.attributes.name || id,
        icon: parsed.attributes.icon || '📝',
        description: parsed.attributes.description || '',
        order: parsed.attributes.order || 999,
        content: parsed.body.trim()
    };
});

// Sort by order
prompts.sort((a, b) => a.order - b.order);

export default prompts;
