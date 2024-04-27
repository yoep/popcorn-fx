package com.github.yoep.player.popcorn.subtitles.controls;

import com.github.yoep.popcorn.backend.subtitles.model.SubtitleLine;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleText;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import java.util.Arrays;
import java.util.Objects;

@Getter
@ToString
@EqualsAndHashCode(of = {"flag"})
public class TrackFlags {
    public static final TrackFlags NORMAL = new TrackFlags(0);                  // 0000 0000
    public static final TrackFlags ITALIC = new TrackFlags(1);                  // 0000 0001
    public static final TrackFlags BOLD = new TrackFlags(2);                    // 0000 0010
    public static final TrackFlags UNDERLINE = new TrackFlags(4);               // 0000 0100
    public static final TrackFlags OUTLINE = new TrackFlags(8);                 // 0000 1000
    public static final TrackFlags OPAQUE_BACKGROUND = new TrackFlags(16);      // 0001 0000
    public static final TrackFlags SEE_THROUGH_BACKGROUND = new TrackFlags(32); // 0010 0000

    private int flag;

    private TrackFlags(int flag) {
        this.flag = flag;
    }

    /**
     * Create track flags from the given {@link SubtitleLine}.
     *
     * @param line The line to create tracks flags of.
     * @return Returns track flags with the flags for the given {@link SubtitleLine}.
     */
    public static TrackFlags from(SubtitleText line) {
        int flags = 0;

        if (line.isItalic())
            flags += 1;
        if (line.isBold())
            flags += 2;
        if (line.isUnderline())
            flags += 4;

        return new TrackFlags(flags);
    }


    /**
     * Check if this track flags contains the given track.
     * This method uses a bitwise operation on the flag field for verifying the flag.
     *
     * @param flag The flag to check for.
     * @return Returns true if this track flags contains the flag, else false.
     */
    public boolean hasFlag(TrackFlags flag) {
        return (this.flag & flag.getFlag()) == flag.getFlag();
    }

    /**
     * Add the given flag to this track.
     *
     * @param flag The flag to add to this track.
     */
    public void addFlag(TrackFlags flag) {
        // check if this track already contains the flag
        // if so, ignore the add operation
        if (hasFlag(flag))
            return;

        this.flag += flag.getFlag();
    }

    /**
     * Add the given flags to this track.
     * Duplicate flags will be ignored.
     *
     * @param flags The flags to add to this track.
     */
    public void addFlags(TrackFlags[] flags) {
        Objects.requireNonNull(flags, "flags cannot be null");
        Arrays.stream(flags).forEach(this::addFlag);
    }

    /**
     * Remove the given flag from this track.
     *
     * @param flag The flag to remove from this track.
     */
    public void removeFlag(TrackFlags flag) {
        // check if this track contains the flag
        // if not, ignore the remove operation
        if (!hasFlag(flag))
            return;

        this.flag -= flag.getFlag();
    }
}
