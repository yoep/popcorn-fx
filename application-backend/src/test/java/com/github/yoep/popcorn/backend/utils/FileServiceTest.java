package com.github.yoep.popcorn.backend.utils;

import org.apache.commons.io.FileUtils;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.io.File;
import java.io.IOException;
import java.nio.charset.Charset;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

class FileServiceTest {
    private FileService fileService;
    @TempDir
    File workingDir;

    @BeforeEach
    void setUp() {
        fileService = new FileService();
    }

    @Test
    void testGetFile_whenPathIsNull_shouldThrowIllegalArgumentException() {
        assertThrows(IllegalArgumentException.class, () -> fileService.getFile(null), "path cannot be null");
    }

    @Test
    void testGetFile_whenPathIsEmpty_shouldReturnCurrentWorkingDirectory() {
        var expectedResult = System.getProperty("user.dir");

        var result = fileService.getFile("");

        assertEquals(expectedResult, result.getAbsolutePath());
    }

    @Test
    void testGetAbsolutePath_whenPathIsEmpty_shouldReturnTheWorkingDirectoryAbsolutePath() {
        var expectedResult = System.getProperty("user.dir");

        var result = fileService.getAbsolutePath("");

        assertEquals(expectedResult, result);
    }

    @Test
    void testSave_whenPathIsGiven_shouldSaveTheContentsToTheGivenLocation() throws IOException {
        var filename = "lorem.txt";
        var contents = "lorem ipsum dolor estla";
        var expectedPath = workingDir.getAbsolutePath() + File.separator + filename;

        fileService.save(expectedPath, contents);
        var result = FileUtils.readFileToString(new File(expectedPath), Charset.defaultCharset());

        assertEquals(contents, result);
    }
}
