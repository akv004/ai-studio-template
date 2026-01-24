import { useState, useMemo } from 'react';
import { Play, Pause, Mic, Square, Volume2, SkipBack, SkipForward } from 'lucide-react';

/**
 * Audio Page
 * 
 * Features:
 * - Waveform display
 * - Play/Record buttons (mock)
 * - Transcription panel
 */
export function AudioPage() {
    const [isPlaying, setIsPlaying] = useState(false);
    const [isRecording, setIsRecording] = useState(false);
    const [currentTime, setCurrentTime] = useState(0);
    const duration = 11.5;

    // Generate mock waveform data
    const waveformBars = useMemo(() => {
        return Array.from({ length: 100 }, () => Math.random() * 80 + 20);
    }, []);

    const mockTranscription = {
        text: "Hello, this is a test transcription from the AI Studio application. The audio quality is excellent and the speech recognition is working perfectly.",
        segments: [
            { start: 0.0, end: 2.5, text: "Hello, this is a test transcription" },
            { start: 2.5, end: 5.2, text: "from the AI Studio application." },
            { start: 5.2, end: 8.0, text: "The audio quality is excellent" },
            { start: 8.0, end: 11.5, text: "and the speech recognition is working perfectly." },
        ]
    };

    const formatTime = (seconds: number) => {
        const mins = Math.floor(seconds / 60);
        const secs = Math.floor(seconds % 60);
        return `${mins}:${secs.toString().padStart(2, '0')}`;
    };

    return (
        <div className="animate-fade-in h-full flex flex-col">
            {/* Page Header */}
            <div className="page-header">
                <div>
                    <h1 className="page-title">Audio</h1>
                    <p className="page-description">Audio processing and transcription</p>
                </div>
                <div className="flex items-center gap-2">
                    <button
                        className={`btn ${isRecording ? 'bg-red-500 hover:bg-red-600' : 'btn-secondary'}`}
                        onClick={() => setIsRecording(!isRecording)}
                    >
                        {isRecording ? <Square className="w-4 h-4" /> : <Mic className="w-4 h-4" />}
                        {isRecording ? 'Stop' : 'Record'}
                    </button>
                </div>
            </div>

            {/* Main Content */}
            <div className="flex-1 flex flex-col gap-4 mt-4">
                {/* Waveform Panel */}
                <div className="panel">
                    <div className="panel-header">
                        <span className="panel-title">Waveform</span>
                        <div className="flex items-center gap-2">
                            <Volume2 className="w-4 h-4 text-[var(--text-muted)]" />
                            <span className="text-sm text-[var(--text-muted)]">audio_sample.wav</span>
                        </div>
                    </div>
                    <div className="p-4">
                        {/* Waveform Visualization */}
                        <div className="waveform-container h-32 bg-[var(--bg-primary)] rounded-lg">
                            {waveformBars.map((height, i) => {
                                const progress = (currentTime / duration) * 100;
                                const barProgress = (i / waveformBars.length) * 100;
                                const isPlayed = barProgress < progress;

                                return (
                                    <div
                                        key={i}
                                        className="waveform-bar"
                                        style={{
                                            height: `${height}%`,
                                            opacity: isPlayed ? 1 : 0.4
                                        }}
                                    />
                                );
                            })}
                        </div>

                        {/* Transport Controls */}
                        <div className="flex items-center justify-center gap-4 mt-4">
                            <button className="btn btn-ghost btn-icon">
                                <SkipBack className="w-5 h-5" />
                            </button>
                            <button
                                className="btn btn-primary w-12 h-12 rounded-full"
                                onClick={() => setIsPlaying(!isPlaying)}
                            >
                                {isPlaying ? (
                                    <Pause className="w-5 h-5" />
                                ) : (
                                    <Play className="w-5 h-5 ml-0.5" />
                                )}
                            </button>
                            <button className="btn btn-ghost btn-icon">
                                <SkipForward className="w-5 h-5" />
                            </button>
                        </div>

                        {/* Time Display */}
                        <div className="flex items-center justify-between mt-4 text-sm text-[var(--text-muted)]">
                            <span>{formatTime(currentTime)}</span>
                            <div className="flex-1 mx-4">
                                <input
                                    type="range"
                                    min="0"
                                    max={duration}
                                    value={currentTime}
                                    onChange={(e) => setCurrentTime(parseFloat(e.target.value))}
                                    className="w-full"
                                />
                            </div>
                            <span>{formatTime(duration)}</span>
                        </div>
                    </div>
                </div>

                {/* Transcription Panel */}
                <div className="panel flex-1">
                    <div className="panel-header">
                        <span className="panel-title">Transcription</span>
                        <span className="status-pill status-success">
                            <span className="status-dot" />
                            Complete
                        </span>
                    </div>
                    <div className="panel-content">
                        <div className="space-y-3">
                            {mockTranscription.segments.map((segment, i) => (
                                <div
                                    key={i}
                                    className="flex gap-4 p-3 rounded-md hover:bg-[var(--bg-tertiary)] cursor-pointer transition-colors"
                                >
                                    <span className="text-xs text-[var(--text-muted)] font-mono w-20">
                                        {formatTime(segment.start)} - {formatTime(segment.end)}
                                    </span>
                                    <span className="flex-1">{segment.text}</span>
                                </div>
                            ))}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
