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
let isSpeaking = false;   // 是否正在说话
let currentLipValue = 0;  // 当前嘴巴开合度 (用于平滑过渡)

// @ts-ignore
config.cubism4.maskSize = 4096;
// @ts-ignore
config.cubism4.renderTextureCount = 1;
// @ts-ignore
if (Live2DModel.config) {
    // @ts-ignore
    Live2DModel.config.maxMasks = 128;
}

// 性能模式：使用中等精度着色器 (通常肉眼难辨区别，由于您之前反馈过白边问题，降低精度有时反而能“模糊”掉瑕疵)
// @ts-ignore
PIXI.Program.defaultFragmentPrecision = PIXI.PRECISION.MEDIUM;
// 保持高流畅度：继续跟随屏幕高刷 (240fps上限)
PIXI.Ticker.shared.maxFPS = 240;
// 性能模式：使用原生分辨率 (不强制2倍超采样)，大幅降低显卡负载
// @ts-ignore
PIXI.Filter.defaultResolution = window.devicePixelRatio || 1;

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
            // 恢复标准混合模式
            premultipliedAlpha: true,
            powerPreference: 'default', // 允许系统自动调度显卡 (省电)
            resolution: window.devicePixelRatio || 1, // 恢复原生分辨率
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
                set('Param66', 1.0);       // 手柄显示开关 (1.0 显示, 0.0 隐藏)
                set('Param61', 0.0);       // 手型：伸展 (0~1)
                set('Param62', 1.0);       // 手型：收缩 (0~1)

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

                // 🟢 平滑口型逻辑
                if (isSpeaking) {
                    // 模拟自然说话的随机张合
                    const speed = 8;
                    const noise = Math.sin(now * speed) * Math.sin(now * speed * 0.5);

                    let targetOpenness = (noise + 1) / 2;
                    targetOpenness = targetOpenness * 0.8 + 0.2;
                    if (Math.random() > 0.95) targetOpenness = 0;
                    currentLipValue += (targetOpenness - currentLipValue) * 0.1;

                    // 🎀 说话时只增加一点点身体活力，但不要干扰鼠标跟随
                    // 我们给身体角度叠加一个很小的随机偏移，而不是覆盖它
                    // 注意：set函数的第三个参数是权重，为了不覆盖鼠标追踪，我们需要更高级的操作
                    // 但这里 pixi-live2d-display 的 setParameterValueById 设置权重 1.0 会覆盖
                    // 所以为了保证鼠标跟随，我们 *不要* 在这里 set 身体和头的角度
                    // 让原生的 autoInteract 去控制它们

                } else {
                    currentLipValue += (0 - currentLipValue) * 0.1;
                }
                set('ParamMouthOpenY', currentLipValue);
            }

        });

        // 点击切换说话状态 (测试用)
        window.addEventListener('mousedown', (e) => {
            if (e.button === 0) {
                // 左键拖拽
                getCurrentWindow().startDragging().catch(() => { });
            } else if (e.button === 2) {
                // 右键切换说话
                isSpeaking = !isSpeaking;
                console.log(isSpeaking ? "开始说话..." : "停止说话");
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
