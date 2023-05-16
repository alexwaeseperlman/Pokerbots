exports.handler = async function (event, context) {
  const AWS = require("aws-sdk");
  const s3 = new AWS.S3();
  const batch = new AWS.Batch();
  // check if file is under the maximum size
  const maxFileSize = process.env.BOT_SIZE;
  const fileSize = event.Records[0].s3.object.size;
  if (fileSize > maxFileSize) {
    console.log("File size too large");
    s3.deleteObject({
      Bucket: event.Records[0].s3.bucket.name,
      Key: event.Records[0].s3.object.key,
    });
    return;
  }
};
