package com.github.yoep.popcorn.ui.stage;

import com.github.spring.boot.javafx.stage.BorderlessStageWrapper;
import lombok.Data;
import lombok.Setter;

@Data
public class BorderlessStageHolder {
    private static final BorderlessStageHolder holder = new BorderlessStageHolder();

    @Setter
    private BorderlessStageWrapper stageWrapper;

    private BorderlessStageHolder() {
    }

    public static BorderlessStageWrapper getWrapper() {
        return holder.getStageWrapper();
    }

    public static void setWrapper(BorderlessStageWrapper wrapper) {
        holder.setStageWrapper(wrapper);
    }
}
