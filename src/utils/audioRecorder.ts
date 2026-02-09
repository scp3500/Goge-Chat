export class AudioRecorder {
    private audioContext: AudioContext | null = null;
    private mediaStream: MediaStream | null = null;
    private processor: ScriptProcessorNode | null = null;
    private source: MediaStreamAudioSourceNode | null = null;
    private samples: Float32Array[] = [];
    private sampleRate: number = 16000; // Sherpa-onnx typically uses 16k

    async start() {
        console.log("[AudioRecorder] Requesting microphone access...");
        this.samples = [];

        // 1. Request access first to get the stream and see what the browser gives us
        // enhancing with constraints for better speech recognition
        this.mediaStream = await navigator.mediaDevices.getUserMedia({
            audio: {
                channelCount: 1,
                echoCancellation: true,
                noiseSuppression: true,
                autoGainControl: true,
                sampleRate: 16000 // Try to request 16k, but browser might ignore
            }
        });

        const track = this.mediaStream.getAudioTracks()[0];
        const settings = track.getSettings();
        console.log(`[AudioRecorder] Microphone access granted. Label: ${track.label}, SampleRate: ${settings.sampleRate}, Channels: ${settings.channelCount}`);

        // 2. Create AudioContext
        // We try to force 16k, but if the browser/hardware doesn't support it, 
        // we must accept what comes and report it correctly.
        const AudioContextClass = window.AudioContext || (window as any).webkitAudioContext;
        this.audioContext = new AudioContextClass({
            sampleRate: 16000,
        });

        // Update our internal tracker to the *actual* rate the context is running at
        this.sampleRate = this.audioContext.sampleRate;
        console.log(`[AudioRecorder] AudioContext created. Actual SampleRate: ${this.sampleRate}Hz`);

        this.source = this.audioContext.createMediaStreamSource(this.mediaStream);

        // ScriptProcessor is deprecated but widely supported in browsers. 
        // AudioWorklet is better but requires a separate file/module.
        this.processor = this.audioContext.createScriptProcessor(4096, 1, 1);

        this.processor.onaudioprocess = (e) => {
            const inputData = e.inputBuffer.getChannelData(0);
            // Only log once in a while to avoid flooding
            if (this.samples.length % 50 === 0) {
                console.log(`[AudioRecorder] Capturing chunks... (current count: ${this.samples.length})`);
            }
            this.samples.push(new Float32Array(inputData));
        };

        this.source.connect(this.processor);
        this.processor.connect(this.audioContext.destination);
        console.log("[AudioRecorder] Processing graph started.");
    }

    async stop(): Promise<{ samples: number[], sampleRate: number }> {
        if (this.processor) {
            this.processor.disconnect();
            this.processor.onaudioprocess = null;
        }
        if (this.source) {
            this.source.disconnect();
        }
        if (this.mediaStream) {
            this.mediaStream.getTracks().forEach(track => track.stop());
        }
        if (this.audioContext) {
            await this.audioContext.close();
        }

        // Flatten samples
        const totalLength = this.samples.reduce((acc, s) => acc + s.length, 0);
        const flattened = new Float32Array(totalLength);
        let offset = 0;
        for (const s of this.samples) {
            flattened.set(s, offset);
            offset += s.length;
        }

        return {
            samples: Array.from(flattened),
            sampleRate: this.sampleRate // This is now updated in start() to be audioContext.sampleRate
        };
    }
}
