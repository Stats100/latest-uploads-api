import express from 'express'
import cors from 'cors'

const app = express()
app.use(cors())

app.get('/get/:id', async (req, res) => {
	try {
		const id = req.params.id.replace('UC', 'UU');
		const response = await fetch(`https://youtube.googleapis.com/youtube/v3/playlistItems?part=snippet&playlistId=${id}&key=${process.env.YOUTUBE_API_KEY}&maxResults=5`);

		if (!response.ok) {
			throw new Error(`Error: ${response.statusText}`);
		}

		const data = await response.json();

		if (!data.items) {
			return res.status(404).json({ error: 'No videos found' });
		}

		const videos = data.items.map((item: { snippet: { resourceId: { videoId: string }; title: string } }) => {
			return {
				videoId: item.snippet.resourceId.videoId,
				title: item.snippet.title
			};
		});

		res.status(200).json(videos);
	} catch (error) {
		res.status(500).json({ error: true, message: error });
	}
});


app.listen(process.env.PORT, () => {
	console.log(`Listening on http://localhost:${process.env.PORT}`)
})