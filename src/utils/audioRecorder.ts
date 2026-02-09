export class AudioRecorder {
    private audioContext: AudioContext | null = null;
    private mediaStream: MediaStream | null = null;
    private processor: ScriptProcessorNode | null = null;
    private source: MediaStreamAudioSourceNode | null = null;
    private samples: Float32Array[] = [];
    private sampleRate: number = 16000; // Sherpa-onnx typically uses 16k

    async start() {
        this.samples = [];
        this.audioContext = new (window.AudioContext || (window as any).webkitAudioContext)({
            sampleRate: this.sampleRate,
        });

        this.mediaStream = await navigator.mediaDevices.getUserMedia({ audio: true });
        this.source = this.audioContext.createMediaStreamSource(this.mediaStream);

        // ScriptProcessor is deprecated but widely supported in browsers. 
        // AudioWorklet is better but requires a separate file/module.
        this.processor = this.audioContext.createScriptProcessor(4096, 1, 1);

        this.processor.onaudioprocess = (e) => {
            const inputData = e.inputBuffer.getChannelData(0);
            this.samples.push(new Float32Array(inputData));
        };

        this.source.connect(this.processor);
        this.processor.connect(this.audioContext.destination);
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
            sampleRate: this.sampleRate
        };
    }
}
