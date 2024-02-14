package com.github.yoep.popcorn.backend.lib;

import com.sun.jna.StringArray;
import com.sun.jna.WString;

import java.io.Closeable;

public class WriteOnlyStringArray extends StringArray implements Closeable {
    public WriteOnlyStringArray(String[] strings) {
        super(strings);
    }

    public WriteOnlyStringArray(String[] strings, boolean wide) {
        super(strings, wide);
    }

    public WriteOnlyStringArray(String[] strings, String encoding) {
        super(strings, encoding);
    }

    public WriteOnlyStringArray(WString[] strings) {
        super(strings);
    }

    @Override
    public void read() {
        // no-op
    }
}
