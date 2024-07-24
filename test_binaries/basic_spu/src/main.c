#include "dbg.h"
#include "intrin.h"
#include "command_list.h"
#include "spu/spu.h"
#include "spu/notes.h"

volatile u8 spu_command_buffer_memory[0x200];

void main() {
    MemBuffer command_buffer_membuff;
    command_buffer_membuff.buffer = (volatile u8 *) (((u32)spu_command_buffer_memory + 3) & ~3);
    command_buffer_membuff.length = sizeof(spu_command_buffer_memory) - 4;

    CommandListRecorder recorder;
    init_commandlist_recorder(&recorder, command_buffer_membuff);

    debug_print("recording command list...");

    spu_command_reset_sample_counter(&recorder, 0);

    spu_command_set_mix(&recorder, 0, false, 20000);
    spu_command_set_mix(&recorder, 0, true, 20000);
    spu_command_pitch_set_mode(&recorder, 0, PitchModeConstant);
    spu_command_oscillator_set_waveform(&recorder, 0, WaveformSquare);
    spu_command_envelope_set_attack(&recorder, 0, 400);
    spu_command_envelope_set_decay(&recorder, 0, 400);
    spu_command_envelope_set_sustain(&recorder, 0, 0x4000);
    spu_command_envelope_set_release(&recorder, 0, 0);

    spu_command_relative_wait(&recorder, 5000);

    spu_command_note_on(&recorder, 0, NoteC4);
    spu_command_relative_wait(&recorder, 5000);
    spu_command_envelope_off(&recorder, 0);
    spu_command_relative_wait(&recorder, 5000);

    spu_command_note_on(&recorder, 0, NoteD4);
    spu_command_relative_wait(&recorder, 5000);
    spu_command_envelope_off(&recorder, 0);
    spu_command_relative_wait(&recorder, 5000);

    spu_command_note_on(&recorder, 0, NoteE4);
    spu_command_relative_wait(&recorder, 5000);
    spu_command_envelope_off(&recorder, 0);
    spu_command_relative_wait(&recorder, 5000);

    spu_command_note_on(&recorder, 0, NoteF4);
    spu_command_relative_wait(&recorder, 5000);
    spu_command_envelope_off(&recorder, 0);
    spu_command_relative_wait(&recorder, 5000);

    spu_command_note_on(&recorder, 0, NoteG4);
    spu_command_relative_wait(&recorder, 5000);
    spu_command_envelope_off(&recorder, 0);
    spu_command_relative_wait(&recorder, 5000);

    spu_command_note_on(&recorder, 0, NoteF4);
    spu_command_relative_wait(&recorder, 5000);
    spu_command_envelope_off(&recorder, 0);
    spu_command_relative_wait(&recorder, 5000);

    spu_command_note_on(&recorder, 0, NoteE4);
    spu_command_relative_wait(&recorder, 5000);
    spu_command_envelope_off(&recorder, 0);
    spu_command_relative_wait(&recorder, 5000);

    spu_command_note_on(&recorder, 0, NoteD4);
    spu_command_relative_wait(&recorder, 5000);
    spu_command_envelope_off(&recorder, 0);
    spu_command_relative_wait(&recorder, 5000);

    spu_command_note_on(&recorder, 0, NoteC4);
    spu_command_relative_wait(&recorder, 10000);
    spu_command_envelope_off(&recorder, 0);
    spu_command_relative_wait(&recorder, 5000);

    spu_command_envelope_mute(&recorder, 0);
    spu_command_stop(&recorder);

    CommandList command_list = finish_commandlist_recorder(&recorder);

    debug_print("submitting command list...");

    volatile u32 spu_submission_completion = 0;
    spu_submit_commandlist(SpuQueue0, &spu_submission_completion, &spu_submission_completion);

    debug_print("command list submission called!");

    while (!spu_submission_completion) {}

    debug_print("command list submitted! running spu...");

    spu_set_sample_rate(SampleRate48000);
    spu_run();

    while(spu_running()) {};

    debug_print("spu finished running");

    while(1) { wfi(); }

}
