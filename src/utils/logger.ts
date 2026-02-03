/**
 * 🚀 Antigravity Frontend Logger
 * A beautiful, styled logger for tracking chat steps and performance.
 */

export class Logger {
    private static prefix = '🚀 [Antigravity]';

    static info(step: string, details?: any) {
        console.log(
            `%c${this.prefix} %c${step}`,
            'color: #6366f1; font-weight: bold;',
            'color: #3b82f6; font-weight: 500;',
            details || ''
        );
    }

    static success(step: string, duration?: number, details?: any) {
        const durationStr = duration !== undefined ? ` (${duration}ms)` : '';
        console.log(
            `%c${this.prefix} %c✅ ${step}${durationStr}`,
            'color: #6366f1; font-weight: bold;',
            'color: #10b981; font-weight: bold;',
            details || ''
        );
    }

    static stage(name: string) {
        console.log(
            `%c${this.prefix} %c────────── ${name.toUpperCase()} ──────────`,
            'color: #6366f1; font-weight: bold;',
            'color: #8b5cf6; font-weight: bold; text-decoration: underline;'
        );
    }

    static context(data: any) {
        console.groupCollapsed(`%c${this.prefix} %c📦 CURRENT CONTEXT`, 'color: #6366f1; font-weight: bold;', 'color: #f59e0b; font-weight: bold;');
        console.table(data);
        console.groupEnd();
    }

    static timing(label: string, duration: number) {
        const color = duration > 1000 ? '#ef4444' : (duration > 500 ? '#f59e0b' : '#10b981');
        console.log(
            `%c${this.prefix} %c⏱️ ${label}: %c${duration}ms`,
            'color: #6366f1; font-weight: bold;',
            'color: #94a3b8;',
            `color: ${color}; font-weight: bold;`
        );
    }

    static error(msg: string, err?: any) {
        console.error(
            `%c${this.prefix} %c❌ ${msg}`,
            'color: #6366f1; font-weight: bold;',
            'color: #ef4444; font-weight: bold;',
            err || ''
        );
    }
}
