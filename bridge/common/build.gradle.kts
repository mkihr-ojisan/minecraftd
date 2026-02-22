import com.google.protobuf.gradle.*

plugins {
    id("com.google.protobuf") version "0.9.6"
}

val protobufVersion = "4.33.5"

dependencies {
    implementation("com.kohlschutter.junixsocket:junixsocket-core:2.10.1")
    implementation("com.google.protobuf:protobuf-kotlin:$protobufVersion")
}

protobuf {
    generateProtoTasks {
        all().configureEach {
            builtins {
		        create("kotlin")
            }
        }
    }
}
