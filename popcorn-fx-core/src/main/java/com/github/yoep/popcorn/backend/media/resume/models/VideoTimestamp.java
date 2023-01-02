package com.github.yoep.popcorn.backend.media.resume.models;

import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Data;
import lombok.NoArgsConstructor;

import javax.validation.constraints.NotNull;
import java.util.Optional;

@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class VideoTimestamp {
    /**
     * The ID of the resumable video.
     */
    private String id;
    /**
     * The last known url of the resumable video.
     */
    @NotNull
    private String filename;
    /**
     * The last known timestamp of the video before it was closed.
     */
    private long lastKnownTime;

    public Optional<String> getId() {
        return Optional.ofNullable(id);
    }
}
