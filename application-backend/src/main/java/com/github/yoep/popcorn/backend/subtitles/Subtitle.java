package com.github.yoep.popcorn.backend.subtitles;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle.Cue;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle.Info;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle.Language;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleCue;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;

import java.io.File;
import java.util.List;
import java.util.Optional;

/**
 * The subtitle contains the parsed information of a subtitle file.
 * This is effectively a wrapper around the {@link SubtitleCue} objects which contain the actual parsed data and a reference to the original
 * {@link SubtitleInfo} from which this {@link Subtitle} was generated.
 */
public record Subtitle(com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle proto) {
    public String filePath() {
        return proto.getFilePath();
    }
    
    public List<Cue> cues() {
        return proto.getCuesList();
    }

    /**
     * Check if this subtitle is the special "none" subtitle.
     *
     * @return Returns true if this subtitle is the "none" subtitle, else false.
     */
    public boolean isNone() {
        return getSubtitleInfo()
                .map(e -> e.getLanguage() == Language.NONE)
                .orElse(false);
    }

    /**
     * Get the subtitle info of this subtitle.
     *
     * @return Returns the subtitle info if present, else {@link Optional#empty()}.
     */
    public Optional<Info> getSubtitleInfo() {
        return Optional.ofNullable(proto.getInfo());
    }

    public File getFile() {
        return new File(proto.getFilePath());
    }
}
