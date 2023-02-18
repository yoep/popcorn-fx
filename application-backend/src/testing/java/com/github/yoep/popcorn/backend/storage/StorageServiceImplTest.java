package testing.java.com.github.yoep.popcorn.backend.storage;

import com.fasterxml.jackson.databind.JsonMappingException;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.github.yoep.popcorn.backend.BackendConstants;
import lombok.AllArgsConstructor;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import org.apache.commons.io.FileUtils;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.junit.jupiter.api.io.TempDir;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.io.File;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class StorageServiceImplTest {
    @Mock
    private ObjectMapper objectMapper;
    @InjectMocks
    private StorageServiceImpl service;
    @TempDir
    File workingDir;

    @Test
    void testDetermineDirectoryWithinStorage_whenDirectoryLeavesTheStorageDirectory_shouldThrowStorageException() {
        System.setProperty(BackendConstants.POPCORN_HOME_PROPERTY, workingDir.getAbsolutePath());

        assertThrows(StorageException.class, () -> service.determineDirectoryWithinStorage("../my-dir"), "Directory is invalid as it leaves the storage space");
    }

    @Test
    void testDetermineDirectoryWithinStorage_whenDirectoryIsValid_shouldReturnExpectedFile() {
        var directory = "my-sub-dir";
        var expectedFile = new File(workingDir.getAbsolutePath() + File.separator + directory);
        System.setProperty(BackendConstants.POPCORN_HOME_PROPERTY, workingDir.getAbsolutePath());

        var result = service.determineDirectoryWithinStorage(directory);

        assertEquals(expectedFile, result);
    }

    @Test
    void testDetermineDirectoryWithinStorage_whenDirectoryIsInvalid_shouldTrowStorageException() {
        System.setProperty(BackendConstants.POPCORN_HOME_PROPERTY, workingDir.getAbsolutePath());

        assertThrows(StorageException.class, () -> service.determineDirectoryWithinStorage("\u0000"));
    }

    @Test
    void testRead_whenStorageNameDoesNotExist_shouldReturnEmpty() {
        var name = "non-existing";

        var result = service.read(name, TestType.class);

        assertTrue(result.isEmpty(), "Expected the storage name to not be found");
    }

    @Test
    void testRead_whenStorageNameExists_shouldReturnTheContents() throws IOException {
        var name = "my-file.json";
        var contents = "lorem ipsum dolor";
        var expectedFile = writeContents(name, contents);
        var expectedResult = new TestType(contents);
        when(objectMapper.readValue(expectedFile, TestType.class)).thenReturn(expectedResult);
        System.setProperty(BackendConstants.POPCORN_HOME_PROPERTY, workingDir.getAbsolutePath());

        var result = service.read(name, TestType.class);

        assertTrue(result.isPresent(), "Expected the storage name to have been found");
        assertEquals(expectedResult, result.get());
    }

    @Test
    void testRead_whenStorageNameContentsAreCorrupt_shouldThrowStorageDeserializationException() throws IOException {
        var name = "my-file.json";
        var contents = "lorem ipsum dolor";
        var expectedFile = writeContents(name, contents);
        when(objectMapper.readValue(expectedFile, TestType.class)).thenThrow(mock(JsonMappingException.class));
        System.setProperty(BackendConstants.POPCORN_HOME_PROPERTY, workingDir.getAbsolutePath());

        assertThrows(StorageDeserializationException.class, () -> service.read(name, TestType.class));
    }

    @Test
    void testRead_whenIoExceptionOccurs_shouldThrowStorageException() throws IOException {
        var name = "my-file.json";
        var contents = "lorem ipsum dolor";
        var expectedFile = writeContents(name, contents);
        when(objectMapper.readValue(expectedFile, TestType.class)).thenThrow(mock(IOException.class));
        System.setProperty(BackendConstants.POPCORN_HOME_PROPERTY, workingDir.getAbsolutePath());

        assertThrows(StorageException.class, () -> service.read(name, TestType.class));
    }

    @Test
    void testStore_whenContentsAreGiven_shouldStoreTheContentsSerialized() throws IOException {
        var name = "my-file.json";
        var contents = "lorem ipsum dolor";
        var serializedContents = "mySerializedContents";
        var expectedFile = new File(workingDir.getAbsolutePath() + File.separator + name);
        when(objectMapper.writeValueAsString(contents)).thenReturn(serializedContents);
        System.setProperty(BackendConstants.POPCORN_HOME_PROPERTY, workingDir.getAbsolutePath());

        service.store(name, contents);

        assertTrue(expectedFile.exists(), "Expected the storage file to have been written");
        var result = FileUtils.readFileToString(expectedFile, StandardCharsets.UTF_8);
        assertEquals(serializedContents, result);
    }

    @Test
    void testStore_whenStorageDirectoryDoesNotExist_shouldCreateStorageDirectory() throws IOException {
        var name = "my-file.json";
        var storageDirectory = "my-storage";
        var storageDirectoryPath = workingDir.getAbsolutePath() + File.separator + storageDirectory;
        var expectedDirectory = new File(storageDirectoryPath);
        when(objectMapper.writeValueAsString(isA(String.class))).thenReturn("");
        System.setProperty(BackendConstants.POPCORN_HOME_PROPERTY, storageDirectoryPath);

        service.store(name, "lorem");

        assertTrue(expectedDirectory.exists(), "Expected the storage directory to have been created");
    }

    @Test
    void testClean_whenNameDoesNotExist_shouldIgnoreAction() {
        System.setProperty(BackendConstants.POPCORN_HOME_PROPERTY, workingDir.getAbsolutePath());

        // this action shouldn't throw any exception
        service.remove("non-existing-file");
    }

    @Test
    void testClean_whenNameExists_shouldRemoveFileFromStorage() {
        var name = "myFile.txt";
        System.setProperty(BackendConstants.POPCORN_HOME_PROPERTY, workingDir.getAbsolutePath());
        service.store(name, "lorem");
        var file = service.retrieve(name)
                .map(Path::toFile)
                .orElseThrow(() -> new StorageException(null, "File was not created"));

        service.remove(name);

        assertFalse(file.exists(), "Expected the file to have been removed");
    }

    private File writeContents(String name, String contents) throws IOException {
        var file = new File(workingDir.getAbsolutePath() + File.separator + name);

        FileUtils.writeStringToFile(file, contents, StandardCharsets.UTF_8);

        return file;
    }

    @ToString
    @EqualsAndHashCode
    @AllArgsConstructor
    public static class TestType {
        private String message;
    }
}