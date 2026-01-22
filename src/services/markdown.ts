// src/services/markdown.ts
import { Marked } from 'marked';
import { markedHighlight } from "marked-highlight";
import hljs from 'highlight.js';
import DOMPurify from 'dompurify';

const customMarked = new Marked(
    markedHighlight({
        langPrefix: 'hljs language-',
        highlight(code, lang) {
            const language = hljs.getLanguage(lang) ? lang : 'plaintext';
            return hljs.highlight(code, { language }).value;
        }
    })
);
customMarked.setOptions({ breaks: true, gfm: true });

// 导出一个干净的渲染函数
export const renderMarkdown = (content: string) => {
    const rawHtml = customMarked.parse(content || '');
    return DOMPurify.sanitize(rawHtml as string);
};