# Guided Discovery Learning App - Merge Sort Pilot

## Core Purpose

A minimal app that enables LLM-guided learning discussions, where students discover merge sort through structured checkpoints while maintaining engagement and natural flow.

## Technical Stack

- Frontend: Rust + egui (immediate mode GUI)
- Backend: Supabase (authentication, storing chat logs)
- LLM: Claude or similar, capable of guiding educational discussions

## Key Features

### 1. Simple Chat Interface

- Two-panel layout:
  - Left: Chat history
  - Right: Current chat input/response
- Clear visual distinction between LLM and student messages
- Timestamp display for messages
- Basic text input with send button

### 2. Hidden Metadata for LLM

Each message includes (invisible to student):

- Timestamp of last LLM message
- Time student started typing
- Time student submitted response
- Current checkpoint number
- Total time spent on current checkpoint

### 3. Progress Tracking

- Current checkpoint indicator (1-6)
- Simple progress bar or checkpoint map
- Time spent on current checkpoint
- Optional: Visual cue if spending too long (>5 min) on checkpoint

### 4. Basic Analytics Capture

Store in Supabase:

- Complete chat logs with timestamps
- Time spent per checkpoint
- Total session duration
- Student identifier (for matching pre/post assessments)

### 5. Session Management

- Simple login (email or student ID)
- Session persistence (can resume if disconnected)
- Clear session start/end points
- Optional: Basic session scheduling

## User Flow

1. Student logs in
2. Sees brief introduction to the learning session
3. LLM initiates discussion about sorting
4. Chat proceeds through checkpoints
5. Session ends with completion message
6. Optional: Brief feedback form

## MVP Constraints

- Text-only interaction (no drawings/diagrams)
- Single linear path through checkpoints
- No multimedia content
- No code execution
- No real-time collaboration
- Limited to merge sort lesson only

## Success Metrics

- Session completion rates
- Time per checkpoint
- Student engagement (response times/lengths)
- Learning outcomes (pre/post assessment)
- Student feedback on experience

## Future Considerations

- Support for different lessons/algorithms
- Drawing/visualization tools
- Code execution environment
- Multiple learning paths
- Peer learning features
- Teacher dashboard
