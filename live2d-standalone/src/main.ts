import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
import * as PIXI from 'pixi.js';
import { Live2DModel, config } from 'pixi-live2d-display';

// ========================================================
// 🟢 【唯一控制按钮】 - 改这个数，Alice 就会变大变小
// ========================================================
const SIZE = 1.25;
// ========================================================

const TARGET_WINDOW_WIDTH = 300 * SIZE;
const TARGET_WINDOW_HEIGHT = 600 * SIZE;

// 内部默认配置 (不需要改)
const ALICE_ZOOM = 1;  // 默认全身展示
const Y_OFFSET = 0.5;     // 默认居中

// @ts-ignore
config.cubism4.maskSize = 4096;
// @ts-ignore
config.cubism4.renderTextureCount = 1;
// @ts-ignore
if (Live2DModel.config) {
    // @ts-ignore
    Live2DModel.config.maxMasks = 128;
}

// 提高着色器精度
// @ts-ignore
PIXI.settings.PRECISION_FRAGMENT = PIXI.PRECISION.HIGH;
PIXI.Ticker.shared.maxFPS = 165;
// @ts-ignore
PIXI.settings.FILTER_RESOLUTION = Math.max(window.devicePixelRatio || 1, 2);

(window as any).PIXI = PIXI;

async function init() {
    console.log('🚀 正在优化执行环境并加载 Alice...');

    // 🟢 立即执行窗口调整，无需等待模型加载
    // 这样一刷新页面，窗口就会立刻变大/变小
    try {
        getCurrentWindow().setSize(new LogicalSize(TARGET_WINDOW_WIDTH, TARGET_WINDOW_HEIGHT));
    } catch (e) {
        console.error('窗口调整失败:', e);
    }

    try {
        const canvas = document.getElementById('canvas') as HTMLCanvasElement;
        const app = new PIXI.Application({
            view: canvas,
            autoStart: true,
            backgroundAlpha: 0,
            resizeTo: window,
            antialias: true,
            powerPreference: 'high-performance',
            resolution: Math.max(window.devicePixelRatio || 1, 2),
            autoDensity: true,
            hello: false
        });

        const modelUrl = '/live2d/alice/alice_model3.json';
        //const modelUrl = '/live2d/hiyori/hiyori.model3.json';
        const model = await Live2DModel.from(modelUrl, {
            autoInteract: true,
            idleMotionGroup: 'Idle'
        });

        if (!model) throw new Error('模型解析失败');

        app.stage.addChild(model as any);
        model.anchor.set(0.5, 0.5);
        (model as any).eventMode = 'static';

        // 监听动作结束，确保 Idle 动作无缝衔接
        (model.internalModel.motionManager as any).on('motionFinish', (group: string) => {
            if (group === 'Idle') {
                (model as any).motion('Idle');
            }
        });

        // 🟢 核心控制区：在这里修改模型的所有动态效果
        // 这里的代码每一帧都会执行，用来让 Alice 动起来
        (model.internalModel as any).on('beforeModelUpdate', () => {
            const coreModel = model.internalModel.coreModel as any;
            if (coreModel) {
                const now = Date.now() / 1000;

                // 辅助函数：简化调用
                // id: 参数名, value: 目标值, weight: 权重(1.0代表强制覆盖)
                const set = (id: string, value: number) => coreModel.setParameterValueById(id, value, 1.0);

                /**
                 * 🎮 [1. 手柄与手部控制] 🎮
                 */
                set('Param66', 0.0);       // 手柄显示开关 (1.0 显示, 0.0 隐藏)
                set('Param61', 1.0);       // 手型：伸展 (0~1)
                set('Param62', 0.0);       // 手型：收缩 (0~1)

                // 摇杆微动：模拟手指不断操作的感觉
                const stickX = Math.sin(now * 3) * 0.2; // 0.2 是幅度，改大摇杆动得更猛
                const stickY = Math.sin(now * 4) * 0.2;
                set('LeftStickX', stickX);      // 左摇杆 X
                set('RightStickX', -stickX);    // 右摇杆 X (反向)
                set('LeftStickY', stickY);      // 左摇杆 Y
                set('RightStickY', stickY);     // 右摇杆 Y

                // 按键动画：随机模拟按键
                //set('ButtonA2', Math.sin(now * 5) > 0.8 ? 1 : 0); // 偶尔按一下 A 键

                /**
                 * 💨 [2. 呼吸与身体动态] 💨
                 */
                // 用于让身体上下起伏的呼吸感
                const breath = Math.sin(now * 1.5) * 0.5 + 0.5;
                set('ParamBreath', breath);     // 基础呼吸参数

                // 手臂摇摆：随着呼吸节奏摆动
                const armSwing = Math.sin(now * 1.2) * 0.25;
                set('Param33', armSwing);       // 左臂摇摆
                set('Param67', -armSwing);      // 右臂摇摆

                // 身体微转：让站姿不那么僵硬
                set('ParamBodyAngleX', Math.sin(now * 0.5) * 2); // 身体轻微左右转
                set('ParamBodyAngleZ', Math.sin(now * 0.7) * 1); // 身体轻微晃动

                /**
                 * 👀 [3. 头部与表情] 👀
                 */
                // set('ParamAngleX', Math.sin(now * 0.3) * 10); // 头部左右摇头 (-30 ~ 30)
                // set('ParamAngleY', Math.sin(now * 0.4) * 5);  // 头部上下点头 (-30 ~ 30)
                // set('ParamAngleZ', Math.sin(now * 0.2) * 5);  // 头部左右歪头 (-30 ~ 30)

                // 眼睛跟随（如果有鼠标交互会自动覆盖这里，这里是待机时的默认值）
                // set('ParamEyeBallX', Math.sin(now) * 0.5); // 眼珠左右移动 (-1 ~ 1)

                /**
                 * ✨ [4. 特效与灯光] ✨
                 */
                const light = Math.sin(now * 0.8) * 0.5 + 0.5;
                set('light', light);       // 全局呼吸灯
                set('Param65', light);     // 附加灯效

                /**
                 * 👗 [5. 物理摆动 (裙子/头发)] 👗
                 * 通常由 physics 物理引擎自动计算，但你也可以手动干预
                 */
                // set('Param17', Math.sin(now * 2) * 0.5); // 裙子 X1 摆动
                // set('Param29', Math.sin(now * 3) * 0.3); // 领带飘动
            }
        });

        const updateLayout = () => {
            const width = window.innerWidth;
            const height = window.innerHeight;

            // 强制检测窗口尺寸 (防止窗口回弹)
            if (Math.abs(width - TARGET_WINDOW_WIDTH) > 2 || Math.abs(height - TARGET_WINDOW_HEIGHT) > 2) {
                getCurrentWindow().setSize(new LogicalSize(TARGET_WINDOW_WIDTH, TARGET_WINDOW_HEIGHT)).catch(() => { });
            }

            app.renderer.resize(width, height);

            // 🟢 稳健缩放算法：优先使用固定画布高度，如果取不到则回退到动态包围盒
            const coreModel = model.internalModel.coreModel as any;

            // 计算缩放比例
            let s = 1.0;
            if (coreModel && coreModel.canvasHeight) {
                // 方案 A: 使用原始画布高度（最准）
                s = (height / coreModel.canvasHeight) * ALICE_ZOOM;
            } else {
                // 方案 B: 降级方案，使用边界框（防止模型变成巨人）
                model.scale.set(1);
                s = (height / model.height) * ALICE_ZOOM;
            }

            model.scale.set(s);
            model.x = width * 0.5;
            model.y = height * Y_OFFSET;
        };

        window.addEventListener('mousedown', (e) => {
            if (e.button === 0) {
                getCurrentWindow().startDragging().catch(() => { });
            }
        });

        window.onresize = updateLayout;
        updateLayout();
        setTimeout(updateLayout, 500);
        setTimeout(updateLayout, 2000);

    } catch (e) {
        console.error('渲染异常:', e);
    }
}

if (document.readyState === 'complete') {
    init();
} else {
    window.onload = init;
}
