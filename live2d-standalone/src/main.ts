import { getCurrentWindow } from '@tauri-apps/api/window';

/* eslint-disable @typescript-eslint/no-explicit-any */
declare const PIXI: any;
const Live2DModel = PIXI.live2d.Live2DModel;

async function init() {
    console.log('Live2D Standalone Initializing...');

    try {
        const canvas = document.getElementById('canvas') as HTMLCanvasElement;
        const app = new PIXI.Application({
            view: canvas,
            autoStart: true,
            backgroundAlpha: 0,
            resizeTo: window,
            resolution: window.devicePixelRatio || 1,
            autoDensity: true,
        });

        // 加载模型
        const modelUrl = '/live2d/hiyori/hiyori_pro_t11.model3.json';
        console.log('Loading model from:', modelUrl);

        const model = await Live2DModel.from(modelUrl);
        app.stage.addChild(model as any);

        model.anchor.set(0.5, 0.5);
        model.interactive = true;

        // 静音
        if (model.internalModel && model.internalModel.motionManager) {
            (model.internalModel.motionManager as any).soundManager = {
                play: () => { },
                stop: () => { },
                volume: 0
            };
        }

        // 拖拽窗口
        model.on('pointerdown', async () => {
            try {
                // @ts-ignore
                if (window.__TAURI__) {
                    const win = getCurrentWindow();
                    await win.startDragging();
                }
            } catch (e) { }
        });

        const updateLayout = () => {
            const width = window.innerWidth;
            const height = window.innerHeight;
            model.x = width / 2;
            model.y = height / 2;
            const scale = Math.min(width / model.width, height / model.height) * 1.2;
            model.scale.set(scale);
        };

        updateLayout();
        window.onresize = updateLayout;

        // 显式展示窗口 (作为主窗口，确保它出来)
        try {
            // @ts-ignore
            if (window.__TAURI__) {
                const win = getCurrentWindow();
                await win.show();
                await win.setFocus();
            }
        } catch (e) { }

    } catch (e) {
        console.error('Live2D Init Error:', e);
    }
}

if (document.readyState === 'complete') {
    init();
} else {
    window.onload = init;
}
