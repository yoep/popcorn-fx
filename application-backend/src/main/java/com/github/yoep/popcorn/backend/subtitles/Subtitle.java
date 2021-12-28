package com.github.yoep.popcorn.backend.subtitles;

import com.github.yoep.popcorn.backend.subtitles.models.SubtitleIndex;
import com.github.yoep.popcorn.backend.subtitles.models.SubtitleInfo;
import javafx.beans.property.SimpleListProperty;
import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import org.springframework.util.Assert;

import java.io.File;
import java.io.Serializable;
import java.util.List;
import java.util.Optional;

/**
 * The subtitle contains the parsed information of a subtitle file.
 * This is effectively a wrapper around the {@link SubtitleIndex} objects which contain the actual parsed data and a reference to the original
 * {@link SubtitleInfo} from which this {@link Subtitle} was generated.
 */
@ToString
@EqualsAndHashCode
public class Subtitle implements Serializable {
    public static final String INDEXES_PROPERTY = "indexes";
    private static final Subtitle NONE = new Subtitle(SubtitleInfo.none());

    private final transient SimpleListProperty<SubtitleIndex> indexes = new SimpleListProperty<>(this, INDEXES_PROPERTY, FXCollections.observableArrayList());
    private final SubtitleInfo subtitleInfo;
    private final File file;

    //region Constructors

    Subtitle(SubtitleInfo subtitleInfo) {
        Assert.notNull(subtitleInfo, "subtitleInfo cannot be null");
        this.subtitleInfo = subtitleInfo;
        this.file = null;
    }

    public Subtitle(File file, List<SubtitleIndex> indexes) {
        Assert.notNull(file, "file cannot be null");
        this.subtitleInfo = null;
        this.file = file;
        this.indexes.addAll(indexes);
    }

    public Subtitle(SubtitleInfo subtitleInfo, File file, List<SubtitleIndex> indexes) {
        Assert.notNull(file, "file cannot be null");
        this.subtitleInfo = subtitleInfo;
        this.file = file;
        this.indexes.addAll(indexes);
    }

    //endregion

    //region Properties

    public ObservableList<SubtitleIndex> getIndexes() {
        return indexes.get();
    }

    public SimpleListProperty<SubtitleIndex> indexesProperty() {
        return indexes;
    }

    public void setIndexes(ObservableList<SubtitleIndex> indexes) {
        this.indexes.set(indexes);
    }

    //endregion

    /**
     * Get the special subtitle none type.
     *
     * @return Returns the special subtitle none.
     */
    public static Subtitle none() {
        return NONE;
    }

    /**
     * Check if this subtitle is the special "none" subtitle.
     *
     * @return Returns true if this subtitle is the "none" subtitle, else false.
     */
    public boolean isNone() {
        return getSubtitleInfo()
                .map(SubtitleInfo::isNone)
                .orElse(false);
    }

    /**
     * Check if this subtitle is a custom subtitle.
     *
     * @return Returns true if this subtitle is a custom subtitle, else false.
     */
    public boolean isCustom() {
        return getSubtitleInfo().isEmpty();
    }

    /**
     * Get the subtitle info of this subtitle.
     *
     * @return Returns the subtitle info if present, else {@link Optional#empty()}.
     */
    public Optional<SubtitleInfo> getSubtitleInfo() {
        return Optional.ofNullable(subtitleInfo);
    }

    public File getFile() {
        return file;
    }
}
