package com.github.yoep.popcorn.backend.media.providers.models;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.sun.jna.Structure;
import lombok.*;

import java.io.Closeable;
import java.io.Serializable;

@Getter
@Builder
@NoArgsConstructor
@AllArgsConstructor
@ToString
@EqualsAndHashCode(callSuper = false)
@JsonIgnoreProperties({"autoAllocate", "stringEncoding", "typeMapper", "fields", "pointer"})
@Structure.FieldOrder({"percentage", "watching", "votes", "loved", "hated"})
public class Rating extends Structure implements Serializable, Closeable {
    public static class ByReference extends Rating implements Structure.ByReference {
    }

    public int percentage;
    public int watching;
    public int votes;
    public int loved;
    public int hated;

    @Override
    public void close() {
        setAutoSynch(false);
    }
}
